use std::fmt;
use crate::parser::extractor::Extraction;
use crate::parser::exceptions::ParseError;

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
    pub jinja_ast: Result<Extraction, ParseError>,
    pub raw_sql: String,
    pub yaml: String,
}

impl fmt::Debug for ModelData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModelData")
            .field("jinja_ast", &self.jinja_ast)
            .field("raw_sql", &self.raw_sql)
            .field("yaml", &self.yaml)
            .finish()
    }
}

impl fmt::Display for ModelData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Jinja AST: {:?}", self.jinja_ast)?;
        writeln!(f, "Raw SQL: {}", self.raw_sql)?;
        writeln!(f, "YAML: {}", self.yaml)?;
        Ok(())
    }
}

impl ModelNode {
    pub fn create(model_name: String, jinja_ast: Result<Extraction, ParseError>, raw_sql: String, yaml: String) -> Self {
        ModelNode {
            model_name,
            data: ModelData {
                jinja_ast,
                raw_sql,
                yaml,
            },
        }
    }
}


