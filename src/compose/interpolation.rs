use regex::Regex;
use std::collections::HashMap;

/// Interpolate ${VAR}, ${VAR:-default}, ${VAR-default}, ${VAR:?error}, ${VAR?error}
/// and $VAR syntax in a string, using the provided environment map.
pub fn interpolate(input: &str, env: &HashMap<String, String>) -> Result<String, String> {
    let re = Regex::new(
        r"(?x)
        \$\$                                    # escaped dollar sign
        |
        \$\{                                    # ${
            ([^}:?-]+)                           # variable name
            (?:
                (:-|:?\?|-|\?)                   # operator
                ([^}]*)                          # default/error value
            )?
        \}                                      # }
        |
        \$([A-Za-z_][A-Za-z0-9_]*)             # $VAR
    ",
    )
    .unwrap();

    let mut result = String::with_capacity(input.len());
    let mut last_end = 0;

    for caps in re.captures_iter(input) {
        let m = caps.get(0).unwrap();
        result.push_str(&input[last_end..m.start()]);

        let matched = m.as_str();
        if matched == "$$" {
            result.push('$');
        } else if let Some(var_name) = caps.get(1) {
            // ${VAR...} syntax
            let name = var_name.as_str();
            let operator = caps.get(2).map(|m| m.as_str());
            let value = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            match operator {
                Some(":-") => {
                    // ${VAR:-default} - use default if unset or empty
                    match env.get(name) {
                        Some(v) if !v.is_empty() => result.push_str(v),
                        _ => result.push_str(value),
                    }
                }
                Some("-") => {
                    // ${VAR-default} - use default if unset
                    match env.get(name) {
                        Some(v) => result.push_str(v),
                        None => result.push_str(value),
                    }
                }
                Some(":?") => {
                    // ${VAR:?error} - error if unset or empty
                    match env.get(name) {
                        Some(v) if !v.is_empty() => result.push_str(v),
                        _ => return Err(format!("Variable '{name}' is not set or empty: {value}")),
                    }
                }
                Some("?") => {
                    // ${VAR?error} - error if unset
                    match env.get(name) {
                        Some(v) => result.push_str(v),
                        None => return Err(format!("Variable '{name}' is not set: {value}")),
                    }
                }
                None => {
                    // ${VAR} - simple substitution
                    if let Some(v) = env.get(name) {
                        result.push_str(v);
                    }
                }
                _ => {
                    result.push_str(matched);
                }
            }
        } else if let Some(var_name) = caps.get(4) {
            // $VAR syntax
            if let Some(v) = env.get(var_name.as_str()) {
                result.push_str(v);
            }
        }

        last_end = m.end();
    }

    result.push_str(&input[last_end..]);
    Ok(result)
}

/// Collect environment variables from the system and .env files
pub fn collect_env(env_files: &[std::path::PathBuf]) -> HashMap<String, String> {
    let mut env: HashMap<String, String> = std::env::vars().collect();

    // Load .env files (later files override earlier ones)
    for path in env_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    env.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    env
}

/// Interpolate all string values in a YAML value recursively
pub fn interpolate_yaml(
    value: &serde_yaml::Value,
    env: &HashMap<String, String>,
) -> Result<serde_yaml::Value, String> {
    match value {
        serde_yaml::Value::String(s) => {
            let interpolated = interpolate(s, env)?;
            Ok(serde_yaml::Value::String(interpolated))
        }
        serde_yaml::Value::Mapping(m) => {
            let mut new_map = serde_yaml::Mapping::new();
            for (k, v) in m {
                let new_key = interpolate_yaml(k, env)?;
                let new_val = interpolate_yaml(v, env)?;
                new_map.insert(new_key, new_val);
            }
            Ok(serde_yaml::Value::Mapping(new_map))
        }
        serde_yaml::Value::Sequence(s) => {
            let mut new_seq = Vec::new();
            for v in s {
                new_seq.push(interpolate_yaml(v, env)?);
            }
            Ok(serde_yaml::Value::Sequence(new_seq))
        }
        other => Ok(other.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_substitution() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        assert_eq!(interpolate("hello $FOO", &env).unwrap(), "hello bar");
        assert_eq!(interpolate("hello ${FOO}", &env).unwrap(), "hello bar");
    }

    #[test]
    fn test_default_value() {
        let env = HashMap::new();
        assert_eq!(
            interpolate("${FOO:-default}", &env).unwrap(),
            "default"
        );
        assert_eq!(
            interpolate("${FOO-default}", &env).unwrap(),
            "default"
        );
    }

    #[test]
    fn test_empty_vs_unset() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "".to_string());
        // :- treats empty as unset
        assert_eq!(interpolate("${FOO:-default}", &env).unwrap(), "default");
        // - only checks if unset
        assert_eq!(interpolate("${FOO-default}", &env).unwrap(), "");
    }

    #[test]
    fn test_error_on_unset() {
        let env = HashMap::new();
        assert!(interpolate("${FOO:?must be set}", &env).is_err());
        assert!(interpolate("${FOO?must be set}", &env).is_err());
    }

    #[test]
    fn test_escaped_dollar() {
        let env = HashMap::new();
        assert_eq!(interpolate("$$FOO", &env).unwrap(), "$FOO");
    }
}
