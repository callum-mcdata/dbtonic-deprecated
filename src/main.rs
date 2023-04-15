use clap::{App, Arg, SubCommand};

mod validation;
mod cli;
mod parser;
mod rules;
mod configuration;

fn main() {

    let matches = App::new("dbtonic")
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
        .get_matches();


    validation::ensure_dbt_project::validate();

    if let Some(_) = matches.subcommand_matches("hello") {
        println!("Hello person, I am dbtonic your friendly neighborhood dbt Connoisseur");
    } else if let Some(evaluate_matches) = matches.subcommand_matches("evaluate") {
        cli::evaluate(evaluate_matches);
    }

    if let Some(get_ast_matches) = matches.subcommand_matches("get-ast") {
        cli::get_ast(get_ast_matches);
    }

    
}