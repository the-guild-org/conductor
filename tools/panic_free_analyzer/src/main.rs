use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use walkdir::WalkDir;

fn is_crate_directory(crate_dir: &Path) -> bool {
  crate_dir.join("Cargo.toml").exists()
}

fn main() -> io::Result<()> {
  let patterns = [
    ("panic", Regex::new(r"panic!\(").unwrap(), "🚨"),
    ("unreachable", Regex::new(r"unreachable!\(").unwrap(), "🚫"),
    ("todo", Regex::new(r"todo!\(").unwrap(), "📝"),
    (
      "unimplemented",
      Regex::new(r"unimplemented!\(").unwrap(),
      "🚧",
    ),
    (
      "array_index",
      Regex::new(r"\w+\s*\[\s*\w+\s*\]").unwrap(),
      "🔢",
    ),
    (
      "expect",
      Regex::new(r"\.expect\(|\.expect_err\(").unwrap(),
      "🔎",
    ),
    ("unwrap", Regex::new(r"\.unwrap\s*\(").unwrap(), "🎁"),
  ];

  let workspace_dir = ".";
  let exclude_crate_name = "panic_free_analyzer";
  let ignored_env_var = std::env::var("IGNORED_CRATES").unwrap_or("".to_string());
  let ignored_crates = ignored_env_var.split(',').collect::<Vec<&str>>();

  let mut crate_counts: HashMap<String, HashMap<&str, (usize, String)>> = HashMap::new();
  let mut total_panic_points = 0;
  let mut expected_annotations: HashMap<String, Vec<String>> = HashMap::new();

  for entry in WalkDir::new(workspace_dir)
    .into_iter()
    .filter_map(|e| e.ok())
  {
    let crate_path = entry.path();

    if is_crate_directory(crate_path) {
      let crate_name = crate_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

      if crate_name.is_empty()
        || crate_name == exclude_crate_name
        || ignored_crates.contains(&crate_name.as_str())
      {
        continue;
      }

      let mut pattern_counts: HashMap<&str, (usize, String)> = HashMap::new();

      for entry in WalkDir::new(crate_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "rs")
        {
          let file = File::open(entry.path())?;
          let reader = io::BufReader::new(file);

          let mut previous_line = String::new();
          for line in reader.lines() {
            let line = line?;
            for (pattern_str, pattern, emoji) in &patterns {
              if pattern.is_match(&line) {
                if previous_line.trim().contains("@expected:") {
                  expected_annotations
                    .entry(crate_name.clone())
                    .or_default()
                    .push(format!(
                      "{}: {}",
                      line,
                      previous_line.trim_start_matches("@expected:").trim()
                    ));
                } else {
                  let count = pattern_counts
                    .entry(pattern_str)
                    .or_insert((0, emoji.to_string()));
                  count.0 += 1;
                  total_panic_points += 1;
                }
              }
            }
            previous_line = line;
          }
        }
      }

      crate_counts.insert(crate_name, pattern_counts);
    }
  }

  println!(
    "# 🚨 Rust Panic Audit: {} Potential Panic Points Detected 🚨\n",
    total_panic_points
  );

  let mut sorted_crates: Vec<(&String, &HashMap<&str, (usize, String)>)> =
    crate_counts.iter().collect();
  sorted_crates.sort_by(|a, b| {
    b.1
      .values()
      .map(|x| x.0)
      .sum::<usize>()
      .cmp(&a.1.values().map(|x| x.0).sum::<usize>())
  });

  for (crate_name, patterns) in sorted_crates {
    let total_usages: usize = patterns.values().map(|x| x.0).sum();
    println!("## Crate: `{}`", crate_name);
    println!("📊 Total Usages: {}\n", total_usages);

    for (pattern, (count, emoji)) in patterns {
      if *count > 0 {
        println!("- {} `{}` usages: {}", emoji, pattern, count);
      }
    }

    if let Some(annotations) = expected_annotations.get(crate_name) {
      println!("\n### 📌 Expected Annotations\n");
      for annotation in annotations {
        println!("- {}", annotation);
      }
    }
    println!();
  }

  Ok(())
}
