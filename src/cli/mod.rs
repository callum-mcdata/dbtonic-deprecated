// General modules
use std::process;

// The cli module
use clap::ArgMatches;

// Multithreading
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::sync::Arc;

// Internal objects
use crate::configuration::dbtonic_config::DbtonicConfig;
use crate::parser::dag::DAG;
use crate::rules::rules_engine::{RulesEngine,RuleResult};

pub fn evaluate(evaluate_matches: &ArgMatches) {
    // Instantiate the DAG
    let dag = DAG::create(evaluate_matches.value_of("model"));

    // Read the config file
    let config = match DbtonicConfig::read() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error reading dbtonic.toml: {:?}", e);
            process::exit(1);
        }
    };

    // Create the RuleRunner
    let rules_engine = RulesEngine::create(&config);

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

pub fn get_ast(get_ast_matches: &ArgMatches) {

    // Initialize the DAG
    let dag = DAG::create(get_ast_matches.value_of("model"));

    // Find the model node for the specified model
    if let Some(model_name) = get_ast_matches.value_of("model") {
        if let Some(model_node) = dag.model_nodes.iter().find(|node| node.model_name == model_name) {
            // Print the AST for the specified model
            println!("AST for model: {}", model_name);
            for stmt in &model_node.data.ast {
                println!("{:#?}", stmt);
            }
        } else {
            eprintln!("Error: Model not found: {}", model_name);
            process::exit(1);
        }
    } else {
        eprintln!("Error: No model specified");
        process::exit(1);
    }
}

pub fn get_tokens(get_tokens_matches: &ArgMatches) {

    // Initialize the DAG
    let dag = DAG::create(get_tokens_matches.value_of("model"));

    // Find the model node for the specified model
    if let Some(model_name) = get_tokens_matches.value_of("model") {
        if let Some(model_node) = dag.model_nodes.iter().find(|node| node.model_name == model_name) {
            // Print the AST for the specified model
            println!("Tokens for model: {}", model_name);
            for tokens in &model_node.data.tokens {
                println!("{:#?}", tokens);
            }
        } else {
            eprintln!("Error: Model not found: {}", model_name);
            process::exit(1);
        }
    } else {
        eprintln!("Error: No model specified");
        process::exit(1);
    }
}