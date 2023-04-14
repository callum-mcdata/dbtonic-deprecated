use crate::rules::rules_engine::{Rule,RuleResult};
use crate::parser::model_node::ModelNode;

pub struct ModelYamlExists;

impl Rule for ModelYamlExists {
    fn name(&self) -> String {
        "yaml_exists".to_string()
    }

    fn description(&self) -> String {
        "The ModelNode must contain data in the yaml property.".to_string()
    }

    fn run(&self, model_node: &ModelNode) -> RuleResult {
        if model_node.data.yaml.is_some() {
            RuleResult::Pass
        } else {
            RuleResult::Fail("The ModelNode does not contain data in the yaml property.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::model_yaml::ModelYaml;
    use crate::parser::model_node::ModelData;

    #[test]
    fn test_yaml_exists_rule_pass() {
        let rule = ModelYamlExists {};

        let model_yaml = ModelYaml {
            name: "test_model".to_string(),
            description: None,
            columns: None,
            tests: None,
            ..Default::default()
        };

        let model_node = ModelNode {
            model_name: "test_model".to_string(),
            data: ModelData {
                ast: vec![],
                tokens: vec![],
                sql: String::new(),
                yaml: Some(model_yaml),
                errors: None
            },
        };

        let result = rule.run(&model_node);
        assert_eq!(result, RuleResult::Pass);
    }

    #[test]
    fn test_yaml_exists_rule_fail() {
        let rule = ModelYamlExists {};

        let model_node = ModelNode {
            model_name: "test_model".to_string(),
            data: ModelData {
                ast: vec![],
                tokens: vec![],
                sql: String::new(),
                yaml: None,
                errors: None
            },
        };

        let result = rule.run(&model_node);
        assert_eq!(
            result,
            RuleResult::Fail("The ModelNode does not contain data in the yaml property.".to_string())
        );
    }

}