use std::path::Path;

pub fn validate() {
    // Checks if a file named dbt_project.yml exists in the current directory. 
    // If it doesn't exist, it prints the error message and exits the program with a non-zero exit code (1).
    // This check will be performed before any dbtonic command is executed.
    if !Path::new("dbt_project.yml").exists() {
        eprintln!("Hey friend, it looks like you're not in a dbt project right now. \
            How about you navigate your way over to a dbt project and give this another shot?");
        std::process::exit(1);
    }
}