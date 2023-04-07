use std::error::Error;
use std::sync::Arc;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;

pub struct ModelValidator {
    rules: Vec<Box<dyn ModelValidationRule>>,
    max_workers: usize,
}