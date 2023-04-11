use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::borrow::Cow;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

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
    pub raw_sql: String,
    pub yaml: String,
}

impl fmt::Debug for ModelData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModelData")
            .field("ast", &self.ast)
            .field("raw_sql", &self.raw_sql)
            .field("yaml", &self.yaml)
            .finish()
    }
}

impl fmt::Display for ModelData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "AST: {:?}", self.ast)?;
        writeln!(f, "Raw SQL: {}", self.raw_sql)?;
        writeln!(f, "YAML: {}", self.yaml)?;
        Ok(())
    }
}

impl ModelNode {
    pub fn create(model_name: String, ast: Vec<Statement>, raw_sql: String, yaml: String) -> Self {
        ModelNode {
            model_name,
            data: ModelData {
                ast,
                raw_sql,
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
        let ast = Parser::parse_sql(&dialect, &sql).unwrap();
    
        let model_node = ModelNode::create(model_name,ast,sql,"".to_string());
    
        return Some(model_node)
    
    }

}


