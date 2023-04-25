use std::path::Path;

pub struct DbtProject;

impl DbtProject {
    pub fn validate(&self) {
        if !Path::new("dbt_project.yml").exists() {
            eprintln!("Hey friend, it looks like you're not in a dbt project right now. \
                How about you navigate your way over to a dbt project and give this another shot?");
            std::process::exit(1);
        }
    }

    pub fn check_dbt_version(&self) {
        let dbt_version_output = std::process::Command::new("dbt")
            .arg("--version")
            .output()
            .expect("Failed to execute 'dbt --version'");

        if dbt_version_output.status.success() {
            println!("dbt installation found");
        } else {
            eprintln!("dbt not found. Please install dbt to use this command.");
            std::process::exit(1);
        }
    }

    pub fn run_dbt_compile(&self) {
        let dbt_compile_output = std::process::Command::new("dbt")
            .arg("compile")
            .output()
            .expect("Failed to execute 'dbt compile'");

        if dbt_compile_output.status.success() {
            println!("dbt compile successful");
            println!("{}", String::from_utf8_lossy(&dbt_compile_output.stdout).trim());
        } else {
            eprintln!("dbt compile failed");
            eprintln!("{}", String::from_utf8_lossy(&dbt_compile_output.stderr).trim());
            std::process::exit(1);
        }
    }
}
