
use std::fmt;
use std::path::{Path, PathBuf};
use glob::glob;
use crate::parser::model_node::ModelNode;
use crate::parser::model_yaml::{ModelYaml, YamlFile};

pub struct DAG {
    pub model_nodes: Vec<ModelNode>,
}

impl DAG {
    pub fn create(model: Option<&str>) -> Self {
        let base_path = std::env::current_dir().unwrap();
        let model_file_paths = Self::get_model_file_paths(model,&base_path);
        let yaml_file_paths = Self::get_yaml_file_paths(model, &base_path);

        let mut model_nodes: Vec<ModelNode> = model_file_paths
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

    fn get_model_file_paths(model: Option<&str>, base_path: &Path) -> Vec<PathBuf> {
        let pattern = match model {
            Some(m) => format!("{}/models/**/{}*.sql", base_path.display(), m),
            None => format!("{}/models/**/*.sql", base_path.display()),
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

    fn get_yaml_file_paths(model: Option<&str>, base_path: &Path) -> Vec<PathBuf> {
        let pattern = match model {
            Some(m) => format!("{}/models/**/{}*.yml", base_path.display(), m),
            None => format!("{}/models/**/*.yml", base_path.display()),
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
            model_node.data.yaml = match model_yamls.iter().find(|m| m.name == model_node.model_name) {
                Some(model_yaml) => Some(model_yaml.to_owned()),
                None => None,
            };
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
        let models_dir = dir.path().join("models");
        fs::create_dir(&models_dir).unwrap();
        let file_path = models_dir.join("test_model.sql");
        fs::write(&file_path, "").unwrap();

        let model_file_paths = DAG::get_model_file_paths(None, dir.path());

        // Check if the test_model.sql file is found
        assert!(model_file_paths.into_iter().any(|path| path == file_path));

        dir.close().unwrap();
    }

    #[test]
    fn test_get_yaml_file_paths() {
    // Create temporary directory for test files
        let dir = tempfile::tempdir().unwrap();
        let models_dir = dir.path().join("models");
        fs::create_dir(&models_dir).unwrap();
        let file_path = models_dir.join("test_yaml.yml"); // Change this line
        fs::write(&file_path, "").unwrap();

        let yaml_file_paths = DAG::get_yaml_file_paths(None, dir.path());

        // Check if the test_yaml.yml file is found
        assert!(yaml_file_paths.into_iter().any(|path| path == file_path));

        dir.close().unwrap();
    }

    #[test]
    fn test_combine_model_nodes_and_yamls() {
        // Read test_model.sql
        let sql_file_path = Path::new("tests/data/test_model.sql");

        // Read test_model.yml
        let yaml_file_path = Path::new("tests/data/test_model.yml");
        let yaml_content = fs::read_to_string(yaml_file_path).unwrap();
        let yaml_file: YamlFile = serde_yaml::from_str(&yaml_content).unwrap();
    
        // Extract ModelYaml instances from YamlFile
        let model_yamls = yaml_file.models;

        let model_node = ModelNode::from_path(sql_file_path.to_path_buf()).unwrap();

        // Combine ModelNode and ModelYaml
        let mut model_nodes = vec![model_node];
        DAG::combine_model_nodes_and_yamls(&mut model_nodes, &model_yamls);

        dbg!(&model_nodes);

        // Check if the ModelYaml was correctly combined with the ModelNode
        assert_eq!(model_nodes[0].data.yaml.as_ref().unwrap().name, "test_model");

    }

}