use crate::parser::model_node::ModelNode;
use crate::configuration::dbtonic_config::DbtonicConfig;
use crate::rules::yml_rules::model_primary_key_tests::UniqueNotNullOrCombinationRule;
use crate::rules::yml_rules::model_yaml_defined::ModelYamlExists;

pub trait Rule: Send + Sync{
    // TODO: Alter this to account for first rule
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn run(&self, model_node: &ModelNode) -> RuleResult;
}

#[derive(Debug, PartialEq)]
pub enum RuleResult {
    Pass,
    Fail(String), // The String holds the error message.
}

pub struct RulesEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl RulesEngine {
    pub fn create(config: &DbtonicConfig) -> Self {
        let mut rules_engine = RulesEngine { rules: Vec::new() };
        rules_engine.add_rules_from_config(config);
        rules_engine
    }
    fn add_rules_from_config(&mut self, config: &DbtonicConfig) {
        if config.rules.unique_not_null_or_combination_rule {
            self.add_rule(Box::new(UniqueNotNullOrCombinationRule {}));
        }

        if config.rules.model_yaml_exists {
            self.add_rule(Box::new(ModelYamlExists {}));
        }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    pub fn run_rules(&self, model_node: &ModelNode) -> Vec<(String, RuleResult)> {
        self.rules
            .iter()
            .map(|rule| {
                let result = rule.run(model_node);
                (rule.name(), result)
            })
            .collect()
    }
}