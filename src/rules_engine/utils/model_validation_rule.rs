use std::error::Error;
use serde_json::Value;

pub trait ValidationRule: Sync {
    fn validate_model(&self, model: &UserConfiguredModel) -> Result<Vec<ValidationIssueType>, Box<dyn Error>>;

    fn validate_model_serialized_for_multiprocessing(&self, serialized_model: &Value) -> Result<String, Box<dyn Error>> {
        let model: UserConfiguredModel = serde_json::from_value(serialized_model.clone())?;
        let issues = self.validate_model(&model)?;
        let results = ModelValidationResults::from_issues_sequence(issues);
        serde_json::to_string(&results).map_err(|err| err.into())
    }
}