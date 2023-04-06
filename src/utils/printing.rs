use std::collections::HashMap;
use colored::Colorize;
use std::path::PathBuf;
use std::borrow::Cow;

pub fn get_model_name(file_name: &str) -> String {
    let file_path = PathBuf::from(file_name);
    let model_path: Cow<'_, str> = match file_path.file_name() {
        Some(name) => name.to_string_lossy().into(),
        None => "".into(),
    };
    let model_name = model_path.trim_end_matches(".sql");
    return model_name.to_string()
}

pub fn print_messages(messages: &HashMap<String, Vec<String>>) {
    for (file_name, message_list) in messages.iter() {
        let model_name = get_model_name(file_name);
        println!("{} {}:", "Model Name:".bold(), model_name.bold().to_string().bright_green());
        for message in message_list {
            println!("{}", message);
        }
    }
}