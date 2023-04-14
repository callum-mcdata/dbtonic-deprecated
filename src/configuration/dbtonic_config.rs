use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Deserialize, PartialEq)]
pub struct DbtonicConfig {
    pub rules: Rules,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Rules {
    pub unique_not_null_or_combination_rule: bool,
    pub model_yaml_exists: bool,
    // Add more rules as I get to them
}

#[derive(Debug)]
pub enum DbtonicConfigError {
    IoError(io::Error),
    TomlError(toml::de::Error),
}

impl From<io::Error> for DbtonicConfigError {
    fn from(error: io::Error) -> Self {
        DbtonicConfigError::IoError(error)
    }
}

impl From<toml::de::Error> for DbtonicConfigError {
    fn from(error: toml::de::Error) -> Self {
        DbtonicConfigError::TomlError(error)
    }
}

impl DbtonicConfig {
    pub fn read() -> Result<Self, DbtonicConfigError> {
        let config_path = Path::new("dbtonic.toml");
        DbtonicConfig::read_from_path(config_path)
    }

    pub fn read_from_path(config_path: &Path) -> Result<Self, DbtonicConfigError> {
        match fs::read_to_string(config_path) {
            Ok(config_str) => {
                let config = toml::from_str(&config_str)?;
                Ok(config)
            }
            Err(_) => Ok(DbtonicConfig::default()),
        }
    }

    // These are the default rules whenever the file is not found
    pub fn default() -> Self {
        DbtonicConfig {
            rules: Rules {
                unique_not_null_or_combination_rule: true,
                model_yaml_exists: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let default_config = DbtonicConfig::default();
        assert_eq!(
            default_config,
            DbtonicConfig {
                rules: Rules {
                    unique_not_null_or_combination_rule: true,
                    model_yaml_exists: true,
                },
            }
        );
    }

    #[test]
    fn test_read_config() {
        let config_str = r#"
[rules]
unique_not_null_or_combination_rule = false
model_yaml_exists = false
"#;

        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("dbtonic.toml");
        let mut file = File::create(&config_path).unwrap();
        file.write_all(config_str.as_bytes()).unwrap();

        let config = DbtonicConfig::read_from_path(&config_path).unwrap();

        assert_eq!(
            config,
            DbtonicConfig {
                rules: Rules {
                    unique_not_null_or_combination_rule: false,
                    model_yaml_exists: false,
                },
            }
        );
    }

}