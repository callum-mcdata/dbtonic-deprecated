// The cli module
use clap::ArgMatches;

// Publishes the ensure dbt project file which contains the validate function
pub mod evaluate_functions;
use crate::cli::evaluate_functions::get_file_paths;
use crate::utils::printing::print_messages;

pub fn evaluate(evaluate_matches: &ArgMatches) {
    let messages = if let Some(model) = evaluate_matches.value_of("model") {
        let file_paths = get_file_paths(Some(model));
        evaluate_functions::evaluate_all_sql_files(file_paths)
    } else {
        let file_paths = get_file_paths(None);
        evaluate_functions::evaluate_all_sql_files(file_paths)
    };

    print_messages(&messages);
}