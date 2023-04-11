use std::fmt;
use crate::parser::model_node::ModelNode;
use crate::utils::directory_operations::get_model_file_paths;

pub struct DAG {
    pub model_nodes: Vec<ModelNode>,
}

impl DAG {
    pub fn create(model: Option<&str>) -> Self {
        let file_paths = get_model_file_paths(model);

        let model_nodes: Vec<ModelNode> = file_paths
            .into_iter()
            .filter_map(|path| ModelNode::from_path(path))
            .collect();

        DAG { model_nodes }
    }
    // to instantiate use
    // let dag = DAG::create(None);

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
    use std::path::PathBuf;

    fn create_test_model_file(file_name: &str, content: &str) -> PathBuf {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join(file_name);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_dag_from_paths_all_models() {
        let model1_path = create_test_model_file("model1.sql", "SELECT * FROM table1");
        let model2_path = create_test_model_file("model2.sql", "SELECT * FROM table2");

        let _ = fs::create_dir("models");
        let _ = fs::hard_link(&model1_path, "models/model1.sql");
        let _ = fs::hard_link(&model2_path, "models/model2.sql");

        let dag = DAG::create(None);

        assert_eq!(dag.model_nodes.len(), 2);

        let _ = fs::remove_dir_all("models");
    }

    #[test]
    fn test_dag_from_paths_specific_model() {
        let model1_path = create_test_model_file("model1.sql", "SELECT * FROM table1");
        let model2_path = create_test_model_file("model2.sql", "SELECT * FROM table2");

        let _ = fs::create_dir("models");
        let _ = fs::hard_link(&model1_path, "models/model1.sql");
        let _ = fs::hard_link(&model2_path, "models/model2.sql");

        let dag = DAG::create(Some("model1"));

        assert_eq!(dag.model_nodes.len(), 1);
        assert_eq!(dag.model_nodes[0].model_name, "model1");

        let _ = fs::remove_dir_all("models");
    }
}