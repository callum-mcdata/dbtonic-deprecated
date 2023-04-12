
use std::fmt;
use std::path::PathBuf;
use glob::glob;
use crate::parser::model_node::ModelNode;
use crate::parser::model_yaml::{ModelYaml, ModelYamls};

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

        let model_metadatas: Vec<ModelYaml> = yaml_file_paths
            .into_iter()
            .filter_map(|path| ModelYamls::from_files(&[path]))
            .flatten()
            .collect();

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


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::fs;
//     use std::path::PathBuf;

//     fn create_test_model_file(file_name: &str, content: &str) -> PathBuf {
//         let temp_dir = tempfile::tempdir().unwrap();
//         let file_path = temp_dir.path().join(file_name);
//         fs::write(&file_path, content).unwrap();
//         file_path
//     }

//     #[test]
//     fn test_dag_from_paths_all_models() {
//         let model1_path = create_test_model_file("model1.sql", "SELECT * FROM table1");
//         let model2_path = create_test_model_file("model2.sql", "SELECT * FROM table2");

//         let _ = fs::create_dir("models");
//         let _ = fs::hard_link(&model1_path, "models/model1.sql");
//         let _ = fs::hard_link(&model2_path, "models/model2.sql");

//         let dag = DAG::create(None);

//         assert_eq!(dag.model_nodes.len(), 2);

//         let _ = fs::remove_dir_all("models");
//     }

//     #[test]
//     fn test_dag_from_paths_specific_model() {
//         let model1_path = create_test_model_file("model1.sql", "SELECT * FROM table1");
//         let model2_path = create_test_model_file("model2.sql", "SELECT * FROM table2");

//         let _ = fs::create_dir("models");
//         let _ = fs::hard_link(&model1_path, "models/model1.sql");
//         let _ = fs::hard_link(&model2_path, "models/model2.sql");

//         let dag = DAG::create(None);

//         assert_eq!(dag.model_nodes.len(), 1);
//         assert_eq!(dag.model_nodes[0].model_name, "model1");

//         let _ = fs::remove_dir_all("models");
//     }
// }