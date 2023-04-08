use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use crate::parser::extractor::{extract_from_source};
use crate::utils::printing::get_model_name;
use crate::rules_engine::ast_rules::contains_source_and_ref::check_source_and_ref;
use crate::rules_engine::ast_rules::contains_multiple_sources::check_multiple_sources;
use crate::rules_engine::ast_rules::contains_no_source_or_ref::check_no_source_or_ref;

pub fn evaluate_all_sql_files(file_paths: Vec<PathBuf>) -> HashMap<String, Vec<String>> {
    let mut messages = HashMap::new();

    for path in file_paths {
        if let Some((model_name, message_list)) = process_sql_file(path.clone()) {
            messages.entry(model_name).or_insert_with(Vec::new).extend(message_list);
        }
    }

    return messages
}

fn process_sql_file(path: PathBuf) -> Option<(String, Vec<String>)> {
    
    let path_str = match path.to_str() {
            Some(s) => s,
            None => return None, // Return early if path can't be converted to string
        };

    let model_name = get_model_name(path_str);

    let sql = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return None, // Return early if file can't be read
    };

    // let model_node = vec![];

    let mut message_list = vec![];
    let mut errors = vec![];

    let jinja_ast = extract_from_source(&sql);

    match jinja_ast {
        Ok(extraction) => {
            if let Some(message) = check_multiple_sources(extraction.clone()) {
                message_list.push(message);
            }

            if let Some(message) = check_source_and_ref(extraction.clone()) {
                message_list.push(message);
            }

            if let Some(message) = check_no_source_or_ref(extraction.clone()) {
                message_list.push(message);
            }

        }
        Err(e) => {
            // Add your custom error message here
            let error_message = format!("Parsing error: {}", e);
            errors.push(error_message);
        }
    }

    if message_list.is_empty() {
        None
    } else {
        Some((model_name, message_list))
    }

}