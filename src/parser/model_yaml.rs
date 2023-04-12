use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Deserialize, Deserializer ,Serialize};
use serde_yaml::Value;
use anyhow::Error;
use serde_yaml::Error as SerdeYamlError;

// This is the grouping of multiple ModelYaml.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ModelYamls {
    pub models: Vec<ModelYaml>,
}

impl ModelYamls {

    pub fn from_files(file_paths: &[&str]) -> Self {
        let mut models = Vec::new();
    
        for file_path in file_paths {
            let result: Result<Vec<ModelYaml>, Error> = (|| {
                let mut file = File::open(Path::new(file_path))?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
    
                let model_yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
                let model_metadata: Vec<ModelYaml> = serde_yaml::from_value(model_yaml["models"].clone())?;
                Ok(model_metadata)
            })();
    
            match result {
                Ok(model_metadata) => models.extend(model_metadata),
                Err(e) => {
                    eprintln!("Error parsing YAML file {}: {}", file_path, e);
                }
            }
        }
    
        ModelYamls { models }
    } 
}

// Replace your existing YamlParseError with this custom error type
#[derive(Debug)]
pub enum YamlParseError {
    Io(std::io::Error),
    SerdeYaml(SerdeYamlError),
}

impl From<std::io::Error> for YamlParseError {
    fn from(error: std::io::Error) -> Self {
        YamlParseError::Io(error)
    }
}

impl From<SerdeYamlError> for YamlParseError {
    fn from(error: SerdeYamlError) -> Self {
        YamlParseError::SerdeYaml(error)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ModelYaml {
    pub name: String,
    pub access: Option<String>,
    pub columns: Option<Vec<ColumnProperties>>,
    pub config: Option<ModelConfigs>,
    pub constraints: Option<Constraints>,
    pub description: Option<String>,
    pub docs: Option<Docs>,
    pub group: Option<String>,
    pub latest_version: Option<f64>,
    pub meta: Option<serde_json::Value>,
    pub tests: Option<Vec<Tests>>,
    pub versions: Option<Vec<Version>>,
}

// In the JSON schema, the quote field can be either a boolean or a Jinja string. 
// Similarly, the tags field can be either a single string or an array of strings. 
// The default deserialization process provided by serde does not cover these 
// specific cases out-of-the-box. By implementing the Deserialize trait for 
// these enums, you can define custom deserialization logic to handle these mixed types.

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ColumnProperties {
    pub name: String,
    pub constraints: Option<Constraints>,
    pub data_type: Option<String>,
    pub description: Option<String>,
    pub meta: Option<serde_json::Value>,
    pub policy_tags: Option<Vec<String>>,
    pub quote: Option<BooleanOrJinjaString>,
    pub tests: Option<Vec<Tests>>,
    pub tags: Option<StringOrArrayOfStrings>,
}

// For BooleanOrJinjaString, the deserialize method checks whether the value is a 
// boolean or a string, and then creates an instance of the BooleanOrJinjaString 
// enum variant accordingly.
#[derive(Debug, Serialize, PartialEq)]
pub enum BooleanOrJinjaString {
    Boolean(bool),
    JinjaString(String),
}

impl<'de> Deserialize<'de> for BooleanOrJinjaString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Bool(b) => Ok(BooleanOrJinjaString::Boolean(b)),
            serde_json::Value::String(s) => Ok(BooleanOrJinjaString::JinjaString(s)),
            _ => Err(serde::de::Error::custom(
                "quote field must be a boolean or a string",
            )),
        }
    }
}

// For StringOrArrayOfStrings, the deserialize method checks whether the value is 
// a single string or an array of strings. If it's an array, it further validates 
// that all elements of the array are strings. Then it creates an instance of the 
// StringOrArrayOfStrings enum variant based on the input value.
#[derive(Debug, Serialize, PartialEq)]
pub enum StringOrArrayOfStrings {
    Single(String),
    Array(Vec<String>),
}

impl<'de> Deserialize<'de> for StringOrArrayOfStrings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) => Ok(StringOrArrayOfStrings::Single(s)),
            serde_json::Value::Array(arr) => {
                let strings: Vec<String> = arr
                    .into_iter()
                    .map(|v| v.as_str().map(String::from))
                    .collect::<Option<Vec<String>>>()
                    .ok_or_else(|| {
                        serde::de::Error::custom("array elements must be strings")
                    })?;
                Ok(StringOrArrayOfStrings::Array(strings))
            }
            _ => Err(serde::de::Error::custom(
                "tags field must be a string or an array of strings",
            )),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ModelConfigs {
    pub contract: Option<Contract>,
    pub grant_access_to: Option<Vec<GrantAccessTo>>,
    pub hours_to_expiration: Option<f64>,
    pub kms_key_name: Option<String>,
    pub labels: Option<std::collections::HashMap<String, String>>,
    pub materialized: Option<String>,
    pub sql_header: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Contract {
    pub enforced: Option<BooleanOrJinjaString>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GrantAccessTo {
    pub database: String,
    pub project: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Constraint {
    pub columns: Option<StringOrArrayOfStrings>,
    pub expression: Option<String>,
    pub name: Option<String>,
    pub constraint_type: String,
    pub warn_unenforced: Option<BooleanOrJinjaString>,
    pub warn_unsupported: Option<BooleanOrJinjaString>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Constraints {
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Docs {
    pub show: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Version {
    pub v: f64,
    pub config: Option<ModelConfigs>,
    pub columns: Option<Vec<Column>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Column {
    IncludeExclude(IncludeExclude),
    ColumnProperties(ColumnProperties),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct IncludeExclude {
    pub include: Option<StringOrArrayOfStrings>,
    pub exclude: Option<StringOrArrayOfStrings>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Tests {
    String(String),
    RelationshipsTest(RelationshipsTest),
    AcceptedValuesTest(AcceptedValuesTest),
    NotNullTest(NotNullTest),
    UniqueTest(UniqueTest),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RelationshipsTest {
    relationships: RelationshipsProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RelationshipsProperties {
    name: Option<String>,
    config: Option<TestConfigs>,
    field: String,
    to: String,
    where_clause: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AcceptedValuesTest {
    accepted_values: AcceptedValuesProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AcceptedValuesProperties {
    name: Option<String>,
    config: Option<TestConfigs>,
    quote: Option<bool>,
    values: Vec<String>,
    where_clause: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct NotNullTest {
    not_null: NotNullProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct NotNullProperties {
    name: Option<String>,
    config: Option<TestConfigs>,
    where_clause: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UniqueTest {
    unique: UniqueProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UniqueProperties {
    name: Option<String>,
    config: Option<TestConfigs>,
    where_clause: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TestConfigs {
    alias: Option<String>,
    database: Option<String>,
    enabled: Option<BooleanOrJinjaString>,
    error_if: Option<String>,
    fail_calc: Option<String>,
    limit: Option<f64>,
    schema: Option<String>,
    severity: Option<Severity>,
    store_failures: Option<BooleanOrJinjaString>,
    tags: Option<StringOrArrayOfStrings>,
    warn_if: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warn,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;
    use std::fs;
    use tempfile::tempdir;

    fn model_yaml() -> ModelYaml {
        ModelYaml {
            name: "stg_customers".to_string(),
            access: None,
            columns: Some(vec![
                ColumnProperties {
                    name: "customer_id".to_string(),
                    constraints: None,
                    data_type: None,
                    description: Some("The unique key for each customer.".to_string()),
                    meta: None,
                    policy_tags: None,
                    quote: None,
                    tests: Some(vec![
                        Tests::String("not_null".to_string()),
                        Tests::String("unique".to_string()),
                    ]),
                    tags: None,
                },
            ]),
            config: None,
            constraints: None,
            description: Some(
                "Customer data with basic cleaning and transformation applied, one row per customer."
                    .to_string(),
            ),
            docs: None,
            group: None,
            latest_version: None,
            meta: None,
            tests: None,
            versions: None,
        }
    }

    #[test]
    fn deserialize_single_model() {
        let yaml = r#"
version: 2
models:
  - name: stg_customers
    description: Customer data with basic cleaning and transformation applied, one row per customer.
    columns:
      - name: customer_id
        description: The unique key for each customer.
        tests:
          - not_null
          - unique
"#;

    let deserialized_model_yaml: ModelYaml = serde_yaml::from_str(yaml).unwrap();
    let expected_model_yaml = model_yaml();

    assert_eq!(deserialized_model_yaml, expected_model_yaml);
    }

    #[test]
    fn test_parse_yaml_files() {
        // Create temporary directory for test files
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("file1.yaml");
        let file2_path = dir.path().join("file2.yaml");

        // Write test YAML content to temporary files
        let file1_content = r#"
models:
  - name: model_1
"#;
        let file2_content = r#"
models:
  - name: model_2
"#;
        fs::write(&file1_path, file1_content).unwrap();
        fs::write(&file2_path, file2_content).unwrap();

        // Call the `from_files` function
        let file_paths = vec![file1_path.to_str().unwrap(), file2_path.to_str().unwrap()];
        let models = ModelYamls::from_files(&file_paths);

        // Check that the parsed models have the expected names
        assert_eq!(models.models.len(), 2);
        assert_eq!(models.models[0].len(), 1);
        assert_eq!(models.models[0].name, "model_1");
        assert_eq!(models.models[1].len(), 1);
        assert_eq!(models.models[1].name, "model_2");

        // Clean up the temporary directory
        dir.close().unwrap();

    }


    #[test]
    fn test_parse_complex_yaml_file() {
        // Create a temporary directory for the test file
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("complex_file.yaml");
        
    // Write test YAML content to the temporary file
    let file_content = r#"
models:
  - name: stg_customers
    description: Customer data with basic cleaning and transformation applied, one row per customer.
    columns:
      - name: customer_id
        description: The unique key for each customer.
        tests:
            - not_null
            - unique

  - name: stg_locations
    description: List of open locations with basic cleaning and transformation applied, one row per location.
    columns:
      - name: location_id
        description: The unique key for each location.
        tests:
            - not_null
            - unique

  - name: stg_order_items
    description: Individual food and drink items that make up our orders, one row per item.
    columns:
      - name: order_item_id
        description: The unique key for each order item.
        tests:
            - not_null
            - unique
"#;

    fs::write(&file_path, file_content).unwrap();

    // Call the `from_files` function
    let file_paths = vec![file_path.to_str().unwrap()];
    let model_yamls = ModelYamls::from_files(&file_paths);


    // Check that the parsed models have the expected names and descriptions
    assert_eq!(model_yamls.models.len(), 1);
    assert_eq!(model_yamls.models[0].models.len(), 3);
    assert_eq!(model_yamls.models[0].models[0].name, "stg_customers");
    assert_eq!(
        model_yamls.models[0].models[0].description.as_deref(),
        Some("Customer data with basic cleaning and transformation applied, one row per customer.")
    );
    assert_eq!(model_yamls.models[0].models[1].name, "stg_locations");
    assert_eq!(
        model_yamls.models[0].models[1].description.as_deref(),
        Some("List of open locations with basic cleaning and transformation applied, one row per location.")
    );
    assert_eq!(model_yamls.models[0].models[2].name, "stg_order_items");
    assert_eq!(
        model_yamls.models[0].models[2].description.as_deref(),
        Some("Individual food and drink items that make up our orders, one row per item.")
    );

    // Check that the parsed columns have the expected names and descriptions
    assert_eq!(model_yamls.models[0].models[0].columns.as_ref().unwrap().len(), 1);
    assert_eq!(
        model_yamls.models[0].models[0].columns.as_ref().unwrap()[0].name,
        "customer_id"
    );
    assert_eq!(
        model_yamls.models[0].models[0].columns.as_ref().unwrap()[0]
            .description
            .as_deref(),
        Some("The unique key for each customer.")
    );

    // Clean up the temporary directory
    dir.close().unwrap();
    }

}