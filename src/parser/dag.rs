
use std::fmt;
use std::path::PathBuf;
use glob::glob;
use crate::parser::model_node::ModelNode;
use crate::parser::model_yaml::{ModelYaml, YamlFile};

pub struct DAG {
    pub model_nodes: Vec<ModelNode>,
}

impl DAG {
    pub fn create(model: Option<&str>) -> Self {
        let model_file_paths = Self::get_model_file_paths(model);
        let yaml_file_paths = Self::get_yaml_file_paths(model);

        let model_nodes: Vec<ModelNode> = model_file_paths
            .into_iter()
            .filter_map(|path| ModelNode::from_path(path))
            .collect();

        let model_yamls: Vec<ModelYaml> = yaml_file_paths
            .into_iter()
            .filter_map(|path| YamlFile::from_file(path).ok())
            .flat_map(|models| models.into_iter())
            .collect();

        Self::combine_model_nodes_and_yamls(&mut model_nodes, &model_yamls);

        DAG { model_nodes }
    }

    fn get_model_file_paths(model: Option<&str>) -> Vec<PathBuf> {
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
        } else {
            //TODO Remove this once I add some watch functions
            println!("{} model file(s) found",file_paths.len())
        }
    
        return file_paths
    
    }

    fn get_yaml_file_paths(model: Option<&str>) -> Vec<PathBuf> {
    
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

    fn combine_model_nodes_and_yamls(model_nodes: &mut Vec<ModelNode>, model_yamls: &Vec<ModelYaml>) {
        for model_node in model_nodes {
            if let Some(model_yaml) = model_yamls.iter().find(|m| m.model_name == model_node.model_name) {
                model_node.data.yaml = model_yaml.clone();
            }
        }
    }

}

impl fmt::Debug for DAG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DAG")
            .field("model_nodes", &self.model_nodes)
            .finish()
    }
}

impl fmt::Display for DAG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DAG:")?;
        for (i, model_node) in self.model_nodes.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, model_node)?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_get_model_file_paths() {
        // Create temporary directory for test files
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_model.sql");
        fs::write(&file_path, "").unwrap();

        let model_file_paths = DAG::get_model_file_paths(None);

        // Check if the test_model.sql file is found
        assert!(model_file_paths.into_iter().any(|path| path == file_path));

        dir.close().unwrap();
    }

    #[test]
    fn test_get_yaml_file_paths() {
        // Create temporary directory for test files
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_yaml.yml");
        fs::write(&file_path, "").unwrap();

        let yaml_file_paths = DAG::get_yaml_file_paths(None);

        // Check if the test_yaml.yml file is found
        assert!(yaml_file_paths.into_iter().any(|path| path == file_path));

        dir.close().unwrap();
    }

}