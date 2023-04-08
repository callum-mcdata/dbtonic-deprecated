use crate::parser::extractor::Extraction;

pub fn check_multiple_sources(ast: Extraction) -> Option<String> {
    let source_count = ast.sources.len();

    if source_count > 1 {
        return Some("\u{274C} This model contains multiple {{ source() }} functions. \
        Only one {{ source() }} function should be used per model.".to_owned())
    } else {
        return None
    }

}