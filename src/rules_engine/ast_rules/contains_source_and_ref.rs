use crate::parser::extractor::Extraction;

pub fn check_source_and_ref(ast: Extraction) -> Option<String> {
    let source_count = ast.sources.len();
    let ref_count = ast.refs.len();

    if source_count > 1 && ref_count > 1 {
        return Some("\u{274C} This model contains both {{ source() }} and {{ ref() }} functions. \
        We highly recommend having a one-to-one relationship between sources and their corresponding staging model, \
        and not having any other model reading from the source. Those staging models are then the ones \
        read from by the other downstream models. This allows renaming your columns and doing minor transformation \
        on your source data only once and being consistent across all the models that will consume the source data.".to_owned())
    } else {
        return None
    }

}