use regex::{Captures, Regex};

type Warnings = Vec<String>;
type Errors = Vec<String>;

pub trait ConductorEnvVars {
    fn get_var(&self, key: &str) -> Option<String>;
}

pub fn interpolate(
    input: &str,
    env_fetcher: impl ConductorEnvVars,
) -> Result<(String, Warnings), Errors> {
    let empty_string = String::with_capacity(0);
    let env_var_interpolation_regex: Regex = Regex::new(
        r"(?x)
        \\\$|\$([[:word:].]+)|\$
        \{([[:word:].]+?)(?::([^}]*))?\}
        ",
    )
    .unwrap();

    let mut errors = Vec::new();
    // Yassin: we can leave this, since we might use it in the future for syntax deprecation
    let warnings = Vec::new();

    let interpolated = env_var_interpolation_regex
        .replace_all(input, |caps: &Captures| {
            if let Some(matched) = caps.get(0) {
                let entire_match = matched.as_str();
                if entire_match == "\\$" {
                    return "$".to_string(); // Return single dollar sign for escaped "\$"
                }
            }

            let var_name = caps.get(2).map_or("", |m| m.as_str());
            let default_value = caps.get(3).map_or(empty_string.as_str(), |m| m.as_str());

            env_fetcher.get_var(var_name).unwrap_or_else(|| {
                if default_value == empty_string {
                    errors.push(format!(
                        "Environment variable `{}` is used in the config file interpolation, but its value was not set, and no default value was provided.",
                        var_name
                    ));
                    empty_string.to_owned()
                } else {
                    default_value.to_owned()
                }
            })
        })
        .to_string();

    if errors.is_empty() {
        Ok((interpolated, warnings))
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockEnvVars {
        vars: HashMap<String, String>,
    }

    impl MockEnvVars {
        fn new() -> Self {
            Self {
                vars: HashMap::new(),
            }
        }

        fn insert(&mut self, key: &str, value: &str) {
            self.vars.insert(key.to_string(), value.to_string());
        }
    }

    impl ConductorEnvVars for MockEnvVars {
        fn get_var(&self, key: &str) -> Option<String> {
            self.vars.get(key).cloned()
        }
    }

    #[test]
    fn should_interpolate_with_set_variable() {
        let mut env_vars = MockEnvVars::new();
        env_vars.insert("API_ENDPOINT", "https://api.example.com/");
        let input = "endpoint: ${API_ENDPOINT}";
        let result = interpolate(input, env_vars).unwrap();
        assert_eq!(result.0, "endpoint: https://api.example.com/");
    }

    #[test]
    fn should_interpolate_with_default_value() {
        let env_vars = MockEnvVars::new();
        let input = "endpoint: ${API_ENDPOINT:https://api.example.com/}";
        let result = interpolate(input, env_vars).unwrap();
        assert_eq!(result.0, "endpoint: https://api.example.com/");
    }

    #[test]
    fn should_interpolate_without_value_or_default() {
        let env_vars = MockEnvVars::new();
        let input = "endpoint: ${API_ENDPOINT}";
        let result = interpolate(input, env_vars);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(&"Environment variable `API_ENDPOINT` is used in the config file interpolation, but its value was not set, and no default value was provided.".to_string()));
    }

    #[test]
    fn should_interpolate_with_escaped_dollar_sign() {
        let env_vars = MockEnvVars::new();
        let input = r"name: \$snaky";
        let result = interpolate(input, env_vars).unwrap();
        assert_eq!(result.0, "name: $snaky");
    }
    #[test]
    fn should_prioritize_environment_variable_over_default_value() {
        let mut env_vars = MockEnvVars::new();
        env_vars.insert("API_ENDPOINT", "https://api.setfromenv.com/");
        let input = "endpoint: ${API_ENDPOINT:https://api.default.com/}";
        let result = interpolate(input, env_vars).unwrap();
        assert_eq!(result.0, "endpoint: https://api.setfromenv.com/");
    }

    #[test]
    fn interpolate_with_multiple_environment_variables() {
        let mut env_vars = MockEnvVars::new();
        env_vars.insert("API_ENDPOINT", "https://api.example.com/");
        env_vars.insert("API_KEY", "12345");

        let input = "endpoint: ${API_ENDPOINT}, key: ${API_KEY}, unused: ${UNUSED_VAR:default}, escaped: \\$escaped_variable";

        let result = interpolate(input, env_vars).unwrap();

        assert_eq!(result.0, "endpoint: https://api.example.com/, key: 12345, unused: default, escaped: $escaped_variable");
    }
}
