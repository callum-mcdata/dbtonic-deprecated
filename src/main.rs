use clap::{App, Arg, SubCommand};

mod validation;
mod cli;
mod parser;
mod rules;

fn main() {

    let matches = App::new("dbtonic")
        .version("0.1.0")
        .author("Callum McCann")
        .about("Your friendly neighborhood build tool Connoisseur")
        .subcommand(SubCommand::with_name("hello")
            .about("Says hello to the user"))
        .subcommand(SubCommand::with_name("evaluate")
            .about("Finds and evaluates a SQL file")
            .arg(Arg::with_name("model")
                .long("model")
                .value_name("FILE")
                .help("Defines the SQL model to evaluate")
                .takes_value(true)))
        .get_matches();

    validation::ensure_dbt_project::validate(); // Add this line to call the function

    if let Some(_) = matches.subcommand_matches("hello") {
        println!("Hello person, I am dbtonic your friendly neighborhood dbt Connoisseur");
    } else if let Some(evaluate_matches) = matches.subcommand_matches("evaluate") {
        cli::evaluate(evaluate_matches);
    }
    
}