pub mod evaluate_functions;
// The cli module
use clap::ArgMatches;

// Multithreading
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::sync::Arc;

// Publishes the ensure dbt project file which contains the validate function
// use crate::utils::directory_operations::get_model_file_paths;
// use crate::utils::printing::print_messages;
use crate::parser::dag::DAG;
use crate::rules::rules_engine::{RulesEngine,RuleResult};
use crate::rules::ast_rules::contains_multiple_sources::ContainsMultipleSources;
use crate::rules::ast_rules::contains_source_and_ref::ContainsSourceAndRef;

// pub fn evaluate(evaluate_matches: &ArgMatches) {
//     let messages = if let Some(model) = evaluate_matches.value_of("model") {
//         let dag = DAG::create(Some(model));
//         let file_paths = get_model_file_paths(Some(model));
//         evaluate_functions::evaluate_all_sql_files(file_paths)
//     } else {
//         let file_paths = get_model_file_paths(None);
//         evaluate_functions::evaluate_all_sql_files(file_paths)
//     };

//     print_messages(&messages);
// }

pub fn evaluate(evaluate_matches: &ArgMatches) {
    // Instantiate the DAG
    let dag = DAG::create(evaluate_matches.value_of("model"));

    // Create the RuleRunner
    let mut rules_engine = RulesEngine::create();
    rules_engine.add_rule(Box::new(ContainsMultipleSources));
    rules_engine.add_rule(Box::new(ContainsSourceAndRef));

    // Run the rules on each of the models in the DAG using multi-threading
    let rules_engine_arc = Arc::new(rules_engine);
    let results: Vec<_> = dag.model_nodes
        .par_iter()
        .map(|model_node| {
            let rule_results = rules_engine_arc.run_rules(model_node);
            (model_node.model_name.clone(), rule_results)
        })
        .collect();

    // Print the results
    for (model_name, rule_results) in results {
        let failed_results: Vec<_> = rule_results.into_iter().filter(|(_, result)| matches!(result, RuleResult::Fail(_))).collect();
    
        if !failed_results.is_empty() {
            println!("Results for model: {}", model_name);
            for (rule_name, result) in failed_results {
                if let RuleResult::Fail(message) = result {
                    println!("  {}: FAIL\n    Reason: {}", rule_name, message);
                }
            }
        }
    }

}