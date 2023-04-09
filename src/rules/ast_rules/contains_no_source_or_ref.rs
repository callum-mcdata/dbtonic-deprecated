use crate::parser::extractor::Extraction;

pub fn check_no_source_or_ref(ast: Extraction) -> Option<String> {
    let source_count = ast.sources.len();
    let ref_count = ast.refs.len();

    //TODO: Add some sort of hard coding regex check here or use some 
    // future AST functionality to see if a table is referenced

    if source_count == 0 && ref_count == 0 {
        return Some("\u{274C} This model contains no sources or refs. This implies \
        that there are hard coded table references in this table, which is not \
        not recommended. Without these functions, dbt has no way of understanding \
        this models place in the DAG.".to_owned())
    } else {
        return None
    }

}