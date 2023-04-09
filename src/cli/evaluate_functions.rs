use std::path::PathBuf;
use std::fs;
use crate::parser::extractor::{extract_from_source};
use crate::utils::printing::get_model_name;
use crate::parser::model_node::{ModelNode};


pub fn create_model_node(path: PathBuf) -> Option<ModelNode> {
    
    let path_str = path.to_str()?;

    let model_name = get_model_name(path_str);

    let sql = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return None, // Return early if file can't be read
    };

    let jinja_ast = extract_from_source(&sql);
    let model_node = ModelNode::create(model_name,jinja_ast,sql,"".to_string());

    return Some(model_node)

}


#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(file_name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("fixtures");
        path.push(file_name);
        path
    }

    #[test]
    fn test_create_model_node_valid_path() {
        let path = fixture_path("valid_sql_file.sql");
        let model_node = create_model_node(path).unwrap();

        assert_eq!(model_node.model_name, "valid_sql_file");
        if let Ok(_) = &model_node.data.jinja_ast {
            assert!(true);
        } else {
            assert!(false, "Expected Extraction, got ParseError");
        }
        assert_eq!(model_node.data.raw_sql.len(), 1);
        assert_eq!(model_node.data.yaml.len(), 0);
    }

}