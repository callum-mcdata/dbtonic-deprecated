use std::path::PathBuf;
use glob::glob;

pub fn get_model_file_paths(model: Option<&str>) -> Vec<PathBuf> {
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
        println!("No model files found.");
    }

    return file_paths

}

pub fn get_yaml_file_paths(model: Option<&str>) -> Vec<PathBuf> {
    
    //TODO: Change this pattern. If it doesn't find a model with the file name 
    // in yml then it should default to parsing all yml files and looking for the
    // model inside one of them. Not sure if that has a significant performance 
    // impact.
    let pattern = match model {
        Some(m) => format!("models/**/{}*.yml", m),
        None => "models/**/*.yml".to_string(),
    };

    let mut file_paths = vec![];

    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        if let Ok(path) = entry {
            file_paths.push(path);
        }
    }

    if file_paths.is_empty() {
        println!("No yml files found.");
    }

    return file_paths

}