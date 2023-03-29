use glob::glob;
use std::path::PathBuf;
use regex::Regex;
use std::fs;
use std::collections::HashMap;
// use crate::parser::parser_functions::parse_sql_file as parse_sql_file;

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
                if let Some(message) = process_sql_file(path.clone()) {
                    let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
                    messages.entry(file_name).or_insert_with(Vec::new).push(message.to_string());
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
    } else {
        for (file_name, message_list) in messages.iter() {
            println!("Messages for file '{}':", file_name);
            for message in message_list {
                println!("{}", message);
            }
        }
    }

    return messages

}

fn process_sql_file(path: PathBuf) -> Option<String> {
    const SOURCE_REGEX: &str = r"\{\{\s*source\s*\(";
    const REF_REGEX: &str = r"\{\{\s*ref\s*\(";
    const MESSAGE_GOOD: &str = "\u{2705} The model {} is all good mate, crack on.";
    const MESSAGE_BAD: &str = "\u{274C} The model {} contains both {{ source() }} and {{ ref() }} functions. \
        We highly recommend having a one-to-one relationship between sources and their corresponding staging model, \
        and not having any other model reading from the source. Those staging models are then the ones \
        read from by the other downstream models. This allows renaming your columns and doing minor transformation \
        on your source data only once and being consistent across all the models that will consume the source data.";
    
    let path_str = match path.to_str() {
            Some(s) => s,
            None => return None, // Return early if path can't be converted to string
        };

    let model_name = path_str.to_owned();

    let sql = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return None, // Return early if file can't be read
    };

    let has_source = regex::Regex::new(SOURCE_REGEX)
        .unwrap()
        .is_match(&sql);
    let has_ref = regex::Regex::new(REF_REGEX)
        .unwrap()
        .is_match(&sql);

    let message = if has_source && has_ref {
        format!("{}, {}", MESSAGE_BAD, model_name)
    } else {
        format!("{}, {}", MESSAGE_GOOD, model_name)
    };

    Some(message)

}