// The cli module
use clap::ArgMatches;

// Publishes the ensure dbt project file which contains the validate function
pub mod evaluate_functions;
use crate::utils::printing::print_messages;

pub fn evaluate(evaluate_matches: &ArgMatches) {
    let messages = if let Some(model) = evaluate_matches.value_of("model") {
        evaluate_functions::evaluate_all_sql_files(Some(model))
    } else {
        evaluate_functions::evaluate_all_sql_files(None)
    };

    print_messages(&messages);
}