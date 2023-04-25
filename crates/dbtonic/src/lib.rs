pub mod validation;
pub mod cli;
pub mod parser;
pub mod rules;
pub mod configuration;

use clap::{App, Arg, SubCommand};
use crate::validation::dbt_project_operations::DbtProject;

pub fn run(args: Vec<String>) {

    let app = App::new("dbtonic")
    .version("0.1.0")
    .author("Callum McCann")
    .about("Your friendly neighborhood build tool Connoisseur")
    .subcommand(SubCommand::with_name("hello")
        .about("Says hello to the user"))
    .subcommand(SubCommand::with_name("evaluate")
        .about("Finds and evaluates a dbt project")
        .arg(Arg::with_name("model")
            .long("model")
            .value_name("FILE")
            .help("Defines the SQL model to evaluate")
            .takes_value(true)))
    .subcommand(SubCommand::with_name("get-ast")
        .about("Returns the AST of a specific model")
        .arg(Arg::with_name("model")
            .long("model")
            .required(true)
            .takes_value(true)
            .help("Defines the SQL model to get AST for")))
    .subcommand(SubCommand::with_name("get-tokens")
        .about("Returns the Tokens of a specific model")
        .arg(Arg::with_name("model")
            .long("model")
            .required(true)
            .takes_value(true)
            .help("Defines the SQL model to get Tokens for")))
    .subcommand(SubCommand::with_name("compile")
        .about("Runs 'dbt compile' in the current directory"))
    ;

    let matches = app.get_matches_from_safe(args).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let dbt_project = DbtProject{};
    DbtProject::validate(&dbt_project);

    if let Some(_) = matches.subcommand_matches("hello") {
        println!("Hello person, I am dbtonic your friendly neighborhood dbt Connoisseur");

    } else if let Some(evaluate_matches) = matches.subcommand_matches("evaluate") {
        cli::evaluate(evaluate_matches);
    }

    if let Some(get_ast_matches) = matches.subcommand_matches("get-ast") {
        cli::get_ast(get_ast_matches);
    }

    if let Some(get_tokens_matches) = matches.subcommand_matches("get-tokens") {
        cli::get_tokens(get_tokens_matches);
    }

    if let Some(_) = matches.subcommand_matches("compile") {
        // Check if dbt is installed
        DbtProject::check_dbt_version(&dbt_project);
    
        // Run 'dbt compile' in the current directory
        DbtProject::run_dbt_compile(&dbt_project);
    }

}
