use glob::glob;
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use crate::parser::extractor::{extract_from_source, Extraction};
use crate::utils::printing::get_model_name;

pub fn get_file_paths(model: Option<&str>) -> Vec<PathBuf> {
    let pattern = match model {
        Some(m) => format!("models/**/{}*.sql", m),
        None => "models/**/*.sql".to_string(),
    };

    let mut file_paths = vec![];

    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        if let Ok(path) = entry {
            file_paths.push(path);
        }
    }

    if file_paths.is_empty() {
        println!("No SQL files found.");
    }

    return file_paths
}

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

    let ast = extract_from_source(&sql);

    let mut message_list = vec![];

    match ast {
        Ok(extraction) => {
            if let Some(message) = check_multiple_sources(extraction.clone()) {
                message_list.push(message);
            }

            if let Some(message) = check_source_and_ref(extraction.clone()) {
                message_list.push(message);
            }
        }
        Err(e) => {
            // Add your custom error message here
            let error_message = format!("Parsing error: {}", e);
            message_list.push(error_message);
        }
    }

    if message_list.is_empty() {
        None
    } else {
        Some((model_name, message_list))
    }

}

fn check_multiple_sources(ast: Extraction) -> Option<String> {
    let source_count = ast.sources.len();

    if source_count > 1 {
        return Some("\u{274C} This model contains multiple {{ source() }} functions. \
        Only one {{ source() }} function should be used per model.".to_owned())
    } else {
        return None
    }

}

fn check_source_and_ref(ast: Extraction) -> Option<String> {
    let source_count = ast.sources.len();
    let ref_count = ast.refs.len();

    if source_count > 1 && ref_count > 1 {
        return Some("\u{274C} This model contains both {{ source() }} and {{ ref() }} functions. \
        We highly recommend having a one-to-one relationship between sources and their corresponding staging model, \
        and not having any other model reading from the source. Those staging models are then the ones \
        read from by the other downstream models. This allows renaming your columns and doing minor transformation \
        on your source data only once and being consistent across all the models that will consume the source data.".to_owned())
    } else {
        return None
    }

}