use lazy_static::lazy_static;
use regex::{Captures, Regex};

// The following file was shamefully copied from the Vector project:
// https://github.com/vectordotdev/vector/blob/master/src/config/vars.rs
// Interpolation is based on:
// https://pubs.opengroup.org/onlinepubs/000095399/basedefs/xbd_chap08.html

lazy_static! {
    pub static ref EMPTY_STRING: String = "".to_string();
    pub static ref ENVIRONMENT_VARIABLE_INTERPOLATION_REGEX: Regex = Regex::new(
        r"(?x)
        \$\$|
        \$([[:word:].]+)|
        \$\{([[:word:].]+)(?:(:?-|:?\?)([^}]*))?\}",
    )
    .unwrap();
}

type Warnings = Vec<String>;
type Errors = Vec<String>;

pub trait ConductorEnvVars {
    fn get_var(&self, key: &str) -> Option<String>;
}

pub fn interpolate(
    input: &str,
    env_fetcher: impl ConductorEnvVars,
) -> Result<(String, Warnings), Errors> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let interpolated = ENVIRONMENT_VARIABLE_INTERPOLATION_REGEX
        .replace_all(input, |caps: &Captures| {
            let flags = caps.get(3).map(|m| m.as_str()).unwrap_or_default();
            let def_or_err = caps.get(4).map(|m| m.as_str()).unwrap_or_default().to_string();

            caps.get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str())
                .map(|name| {
                    let val = env_fetcher.get_var(name);
                    match flags {
                        ":-" => match val {
                            Some(v) if !v.is_empty() => v,
                            _ => def_or_err,
                        },
                        "-" => val.unwrap_or(def_or_err),
                        ":?" => match val {
                            Some(v) if !v.is_empty() => v,
                            _ => {
                                errors.push(format!(
                                    "Non-empty env var required in config. name = {:?}, error = {:?}",
                                    name, def_or_err
                                ));
                                EMPTY_STRING.to_owned()
                            },
                        }
                        "?" => val.unwrap_or_else(|| {
                            errors.push(format!(
                                "Missing env var required in config. name = {:?}, error = {:?}",
                                name, def_or_err
                            ));
                            EMPTY_STRING.to_owned()
                        }),
                        _ => val.unwrap_or_else(|| {
                            warnings
                                .push(format!("Unknown env var in config. name = {:?}", name));
                            EMPTY_STRING.to_owned()
                        }),
                    }
                })
                .unwrap_or("$".to_string())
                .to_string()
        })
        .into_owned();

    if errors.is_empty() {
        Ok((interpolated, warnings))
    } else {
        Err(errors)
    }
}
