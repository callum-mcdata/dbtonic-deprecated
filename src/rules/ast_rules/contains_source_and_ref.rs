use crate::rules::rules_engine::RuleResult;
use crate::rules::rules_engine::Rule;
use crate::parser::model_node::ModelNode;

pub struct ContainsSourceAndRef;

impl Rule for ContainsSourceAndRef {

    fn name(&self) -> String {
        "ContainsSourceAndRef".to_string()
    }

    fn description(&self) -> String {
        "Checks if the model contains both source and ref".to_string()
    }

    fn run(&self, model_node: &ModelNode) -> RuleResult {
        if let Ok(extraction) = &model_node.data.jinja_ast {
            if extraction.sources.len() >= 1 && extraction.refs.len() >= 1 {
                return RuleResult::Fail(
                    "This model contains both {{ source() }} and {{ ref() }} functions.".to_string())
            } else {
                RuleResult::Pass
            }
        } else {
            RuleResult::Fail("Some aspect of the Jinja parsing failed. Please open an issue in the repo!".to_string())
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::model_node::ModelData;
    use crate::parser::extractor::Extraction;
    use crate::parser::exceptions::{ParseError,SourceError};

    fn create_test_model_node(sources: Vec<(String, String)>, refs: Vec<(String, Option<String>)>) -> ModelNode {
        ModelNode {
            model_name: "test".to_string(),
            data: ModelData {
                jinja_ast: Ok(Extraction {
                    sources,
                    refs,
                    configs: vec![],
                    vars: vec![],
                    macros: vec![],
                }),
                raw_sql: "SELECT * FROM {{source('ecom','sales')}} left join {{ref('orders')}};".to_string(),
                yaml: "".to_string(),
            },
        }
    }

    #[test]
    fn test_contains_source_and_ref_rule() {
        let rule = ContainsSourceAndRef;

        let model_node1 = create_test_model_node(vec![("ecom".to_string(), "sales".to_string())], vec![]);
        assert_eq!(rule.run(&model_node1), RuleResult::Pass);

        let model_node2 = create_test_model_node(vec![], vec![("orders".to_string(), None)]);
        assert_eq!(rule.run(&model_node2), RuleResult::Pass);

        let model_node3 = create_test_model_node(
            vec![("ecom".to_string(), "sales".to_string())],
            vec![("orders".to_string(), None)],
        );
        assert_eq!(rule.run(&model_node3), RuleResult::Fail("This model contains both {{ source() }} and {{ ref() }} functions.".to_string()));

        let model_node4 = ModelNode {
            model_name: "test".to_string(),
            data: ModelData {
                jinja_ast: Err(ParseError::SourceE(SourceError::TreeSitterError)),
                raw_sql: "SELECT * FROM {{source('ecom','sales')}} left join {{ref('orders')}};".to_string(),
                yaml: "".to_string(),
            },
        };
        assert_eq!(rule.run(&model_node4), RuleResult::Fail("Some aspect of the Jinja parsing failed. Please open an issue in the repo!".to_string()));
    }

}