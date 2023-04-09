use crate::parser::model_node::ModelNode;

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
    pub fn create() -> Self {
        RulesEngine { rules: Vec::new() }
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