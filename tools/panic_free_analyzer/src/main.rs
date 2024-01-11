use regex::Regex;
use std::collections::{HashMap, HashSet};
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

  let ignored_files_env_var = std::env::var("IGNORED_FILES").unwrap_or_default();
  let ignored_files = ignored_files_env_var
    .split(',')
    .map(|s| s.to_string())
    .collect::<HashSet<String>>();

  let mut crate_counts: HashMap<String, HashMap<&str, (usize, String)>> = HashMap::new();
  let mut expected_annotations: HashMap<String, Vec<(String, String, String)>> = HashMap::new();

  let mut total_actual_audits = 0;

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

      let crate_name = crate_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

      for entry in WalkDir::new(crate_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "rs")
        {
          let file_path = entry.path();
          let relative_file_path = file_path
            .strip_prefix(workspace_dir)
            .unwrap_or(file_path)
            .to_str()
            .unwrap_or_default();

          // skip if the file is in the ignored list
          if ignored_files.contains(&format!("./{}", relative_file_path)) {
            continue;
          }

          let file = File::open(entry.path())?;
          let reader = io::BufReader::new(file);

          let mut line_number = 0;
          let mut expected_next_line = false;
          let mut last_expected_annotation = String::new();

          for line in reader.lines() {
            let line = line?;
            line_number += 1;

            if expected_next_line {
              let location = format!("{}:{}", file_path.display(), line_number);

              expected_annotations
                .entry(crate_name.clone())
                .or_default()
                .push((last_expected_annotation.clone(), line.clone(), location));
              expected_next_line = false;
              continue;
            }

            if line.trim().starts_with("// @expected:") {
              last_expected_annotation = line.trim().to_string();
              expected_next_line = true;
              continue;
            }

            for (pattern_str, pattern, emoji) in &patterns {
              if pattern.is_match(&line) {
                let count = pattern_counts
                  .entry(pattern_str)
                  .or_insert((0, emoji.to_string()));
                if !expected_next_line {
                  count.0 += 1;
                }
              }
            }
          }
        }
      }

      if !pattern_counts.is_empty() || expected_annotations.contains_key(&crate_name) {
        let actual_audits: usize = pattern_counts.values().map(|x| x.0).sum();
        crate_counts.insert(crate_name, pattern_counts);
        total_actual_audits += actual_audits;
      }
    }
  }

  // adjust counts for crates with only 'array_index' errors
  for pattern_counts in crate_counts.values_mut() {
    let only_array_index_errors = pattern_counts
      .iter()
      .all(|(pattern, &(count, _))| *pattern == "array_index" || count == 0);

    if only_array_index_errors {
      if let Some((count, _)) = pattern_counts.get_mut("array_index") {
        total_actual_audits = total_actual_audits - *count;
        *count = 0;
      }
    }
  }

  if total_actual_audits == 0 {
    println!("# 😎 Well Done! No actual audit issues found. 🎉\n");
  } else {
    println!(
      "# 🚨 Rust Panic Audit: {} Potential Panic Points Detected 🚨\n",
      total_actual_audits
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
      if total_usages != 0 {
        println!("## Crate: `{}`", crate_name);
        println!("📊 Total Usages: {}\n", total_usages);

        for (pattern, (count, emoji)) in patterns {
          if *count > 0 {
            println!("- {} `{}` usages: {}", emoji, pattern, count);
          }
        }
      }
    }
  }

  if !expected_annotations.is_empty() {
    println!("\n## 📌 Expected Annotations\n");

    for (crate_name, annotations) in expected_annotations {
      println!("### Crate: `{}`", crate_name);
      println!("📊 Total Expected Usages: {}\n", annotations.len());
      println!(
        "\n<details>
  <summary>expand details</summary>\n",
      );
      let mut index = 0;
      for (annotation, code_line, location) in annotations {
        index += 1;
        println!(
          "{}. Reason: \"{}\"\n- Code: `{}`\n- Location: `{}`\n",
          index,
          annotation.replace("// @expected:", "").trim(),
          code_line.trim(),
          location
        );
      }
      println!("</details>");
      println!();
    }
  }

  Ok(())
}
