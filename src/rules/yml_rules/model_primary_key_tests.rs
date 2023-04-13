use serde_yaml::Value;

use crate::rules::rules_engine::*;
use crate::parser::model_node::ModelNode;
use crate::parser::model_yaml::Tests;

pub struct UniqueNotNullOrCombinationRule;

impl Rule for UniqueNotNullOrCombinationRule {
    fn name(&self) -> String {
        "unique_not_null_or_combination".to_string()
    }

    fn description(&self) -> String {
        "Each model should contain either a single column with the unique and not_null test OR the dbt_utils.unique_combination_of_columns test at the model level.".to_string()
    }

    fn run(&self, model_node: &ModelNode) -> RuleResult {
        let yaml = match &model_node.data.yaml {
            Some(yaml) => yaml,
            None => return RuleResult::Fail("Model does not have an associated YAML".to_string()),
        };

        let mut unique_not_null = false;

        if let Some(columns) = &yaml.columns {
            for column in columns {
                if let Some(tests) = &column.tests {
                    let unique_test = tests.iter().any(|test| match test {
                        Tests::String(s) => s == "unique",
                        _ => false,
                    });
                    let not_null_test = tests.iter().any(|test| match test {
                        Tests::String(s) => s == "not_null",
                        _ => false,
                    });

                    if unique_test && not_null_test {
                        unique_not_null = true;
                        break;
                    }
                }
            }
        }

        if unique_not_null {
            return RuleResult::Pass;
        }

        if let Some(tests) = &yaml.tests {
            let unique_combination_test_key = Value::String("dbt_utils.unique_combination_of_columns".to_string());
            let unique_combination_test = tests.iter().any(|test| match test {
                Tests::CustomTest(value) => value
                    .as_mapping()
                    .map(|map| map.contains_key(&unique_combination_test_key))
                    .unwrap_or(false),
                _ => false,
            });
    
            if unique_combination_test {
                return RuleResult::Pass;
            }
        }

        RuleResult::Fail(
            "The model does not satisfy the unique, not_null, or unique_combination_of_columns requirements."
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::rules::yml_rules::model_primary_key_tests::Tests::NotNullTest;
    use crate::parser::model_yaml::{ModelYaml, NotNullProperties};
    use crate::parser::model_node::ModelData;
    use crate::parser::model_yaml::NotNullTestContents;
    
    #[test]
    fn test_unique_combination_rule() {
        let rule = UniqueNotNullOrCombinationRule {};

        // ModelNode with unique_combination_of_columns test at the model level
        let model_yaml1 = ModelYaml {
            name: "test_model1".to_string(),
            description: None,
            columns: None,
            tests: Some(vec![
                Tests::CustomTest(serde_yaml::from_str("{dbt_utils.unique_combination_of_columns: {combination_of_columns: [id, date]}}").unwrap()),
            ]),
            ..Default::default()
        };

        let model_node1 = ModelNode {
            model_name: "test_model1".to_string(),
            data: ModelData {
                ast: vec![],
                tokens: vec![],
                sql: String::new(),
                yaml: Some(model_yaml1),
            },
        };

        let result1 = rule.run(&model_node1);
        assert_eq!(result1, RuleResult::Pass);

        // ModelNode without unique_combination_of_columns test at the model level
        let model_yaml2 = ModelYaml {
            name: "test_model2".to_string(),
            description: None,
            columns: None,
            // tests: Tests::NotNullTest{not_null: NotNullProperties}
            tests: Some(vec![
                Tests::NotNullTest(NotNullTestContents {
                    not_null: NotNullProperties {
                        name: Some("column_name".to_string()),
                        config: None,
                        where_clause: None,
                    },
                }),
            ]),
            ..Default::default()
        };

        let model_node2 = ModelNode {
            model_name: "test_model2".to_string(),
            data: ModelData {
                ast: vec![],
                tokens: vec![],
                sql: String::new(),
                yaml: Some(model_yaml2),
            },
        };

        let result2 = rule.run(&model_node2);
        assert_ne!(result2, RuleResult::Pass);
    }
}
