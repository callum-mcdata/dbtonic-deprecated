use glob::glob;
use std::path::PathBuf;
use regex::Regex;
use std::fs;
use std::collections::HashMap;
use crate::parser::parser_functions::parse_sql_file as parse_sql_file;

pub fn evaluate_all_sql_files(model: Option<&str>) ->  HashMap<String, Vec<String>>{
    // If select isn't provided to evaluate, this function parses through the local
    // and child directories for any files that match the pattern (ie are sql files).
    // For any file it finds, it runs it through the evaluate sql file function.
    
    let pattern = match model {
        Some(m) => format!("models/**/{}*.sql", m),
        None => "models/**/*.sql".to_string(),
    };

    let mut messages = HashMap::new();

    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if let Some((model_name, message_list)) = process_sql_file(path.clone()) {
                    messages.entry(model_name).or_insert_with(Vec::new).extend(message_list);
                }
            }
            Err(e) => {
                let error_message = format!("Error while processing the file: {:?}", e);
                messages.entry("Error".to_string()).or_insert_with(Vec::new).push(error_message);
            }
        }
    }

    if messages.is_empty() {
        println!("No SQL files found in the current and child directories");
    }

    return messages

}

fn process_sql_file(path: PathBuf) -> Option<(String, Vec<String>)> {
    
    let path_str = match path.to_str() {
            Some(s) => s,
            None => return None, // Return early if path can't be converted to string
        };

    let model_name = path_str.to_owned();

    let sql = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return None, // Return early if file can't be read
    };

    println!("testing string");

    parse_sql_file(sql.as_str());

    let mut message_list = vec![];

    if let Some(message) = check_multiple_sources(&sql) {
        message_list.push(message);
    }

    if let Some(message) = check_source_and_ref(&sql) {
        message_list.push(message);
    }

    if message_list.is_empty() {
        None
    } else {
        Some((model_name, message_list))
    }

}

fn check_multiple_sources(sql: &str) -> Option<String> {
    const SOURCE_REGEX: &str = r"\{\{\s*source\s*\(";
    let source_count = regex::Regex::new(SOURCE_REGEX)
        .unwrap()
        .find_iter(&sql)
        .count();
    if source_count > 1 {
        Some("\u{274C} This model contains multiple {{ source() }} functions. \
        Only one {{ source() }} function should be used per model.".to_owned())
    } else {
        None
    }
}

fn check_source_and_ref(sql: &str) -> Option<String> {
    const SOURCE_REGEX: &str = r"\{\{\s*source\s*\(";
    const REF_REGEX: &str = r"\{\{\s*ref\s*\(";
    const MESSAGE: &str = "\u{274C} This model contains both {{ source() }} and {{ ref() }} functions. \
        We highly recommend having a one-to-one relationship between sources and their corresponding staging model, \
        and not having any other model reading from the source. Those staging models are then the ones \
        read from by the other downstream models. This allows renaming your columns and doing minor transformation \
        on your source data only once and being consistent across all the models that will consume the source data.";

    let has_source = regex::Regex::new(SOURCE_REGEX)
        .unwrap()
        .is_match(&sql);
    let has_ref = regex::Regex::new(REF_REGEX)
        .unwrap()
        .is_match(&sql);

    if has_source && has_ref {
        Some(MESSAGE.to_owned())
    } else {
        None
    }

}