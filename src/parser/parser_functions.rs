use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::fs;

pub fn parse_sql_file(file_path: &str) -> Result<Vec<sqlparser::ast::Statement>, String> {
    // NOTES:
    // 1. We've used map_err() to convert errors from fs::read_to_string() 
    //    and Parser::parse_sql() into Strings, which can be returned as errors.
    // 2. We're using the ? operator to propagate errors from map_err() and Parser::parse_sql()
    // 3. We're returning the AST as a Vec<sqlparser::ast::Statement> instead of
    //    a reference to a Vec<Statement>. This simplifies the return type and avoids 
    //    the need for lifetime annotations.

    let sql = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, &sql).map_err(|e| e.to_string())?;
    Ok(ast)
}