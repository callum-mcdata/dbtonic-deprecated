use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::borrow::Cow;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token,Tokenizer};
use crate::parser::model_yaml::ModelYaml;


pub struct ModelNode {
    pub model_name: String,
    pub data: ModelData,
}

impl fmt::Debug for ModelNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModelNode")
            .field("model_name", &self.model_name)
            .field("data", &self.data)
            .finish()
    }
}

impl fmt::Display for ModelNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ModelNode: {}", self.model_name)?;
        write!(f, "  {}", self.data)?;
        Ok(())
    }
}

// This is the model data struct
pub struct ModelData {
    pub ast: Vec<Statement>,
    pub tokens: Vec<Token>,
    pub sql: String,
    pub yaml: Option<ModelYaml>,
}

impl fmt::Debug for ModelData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModelData")
            .field("ast", &self.ast)
            .field("tokens", &self.tokens)
            .field("sql", &self.sql)
            .field("yaml", &self.yaml)
            .finish()
    }
}

impl fmt::Display for ModelData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "AST: {:?}", self.ast)?;
        writeln!(f, "Tokens: {:?}", self.tokens)?;
        writeln!(f, "SQL: {}", self.sql)?;
        writeln!(f, "YAML: {:?}", self.yaml)?;
        Ok(())
    }
}

impl ModelNode {
    pub fn create(model_name: String, ast: Vec<Statement>, tokens: Vec<Token>, sql: String, yaml: Option<ModelYaml> ) -> Self {
        ModelNode {
            model_name,
            data: ModelData {
                ast,
                tokens,
                sql,
                yaml,
            },
        }
    }
    
    // How to use this function:
    // let model_node = ModelNode::from_path(path)?;
    pub fn from_path(path: PathBuf) -> Option<ModelNode> {
    
        let path_str = path.to_str()?;

        let file_path = PathBuf::from(path_str);
        let model_path: Cow<'_, str> = match file_path.file_name() {
            Some(name) => name.to_string_lossy().into(),
            None => "".into(),
        };
        let model_name = model_path.trim_end_matches(".sql").to_string();    

        let sql = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => return None, // Return early if file can't be read
        };
    
        let dialect = GenericDialect {}; // or AnsiDialect, or your own dialect ...

        let tokens: Vec<Token> = {
            match Tokenizer::new(&dialect, &sql).tokenize() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error tokenizing SQL: {:?}", e);
                    vec![]
                }
            }
        };

        let ast = Parser::parse_sql(&dialect, &sql).unwrap();
    
        let model_node = ModelNode::create(model_name, ast, tokens, sql , None);
    
        return Some(model_node)
    
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_from_path() {
        // Create a temporary file with SQL content for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_model.sql");
        fs::write(&file_path, "SELECT * FROM ( SELECT 1 FROM {{ ref('test_model') }} )").unwrap();

        let model_node = ModelNode::from_path(PathBuf::from(file_path)).unwrap();

        assert_eq!(model_node.model_name, "test_model");
        assert_eq!(model_node.data.sql, "SELECT * FROM ( SELECT 1 FROM {{ ref('test_model') }} )");
        assert!(!model_node.data.ast.is_empty());
        assert!(!model_node.data.tokens.is_empty());
    }

}