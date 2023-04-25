use std::fs::File;
use std::io::Write;

use dbtonic::validation::dbt_project_operations::DbtProject;

fn setup_dbt_project() {
    let mut file = File::create("dbt_project.yml").expect("Failed to create dbt_project.yml");
    file.write_all(b"name: test_project\nversion: '1.0.0'\n")
        .expect("Failed to write to dbt_project.yml");
}

fn cleanup_dbt_project() {
    std::fs::remove_file("dbt_project.yml").expect("Failed to delete dbt_project.yml");
}

#[test]
fn test_validate() {
    setup_dbt_project();

    let dbt_project = DbtProject;
    dbt_project.validate();

    cleanup_dbt_project();
}

#[test]
#[should_panic(expected = "Hey friend, it looks like you're not in a dbt project right now.")]
fn test_validate_no_dbt_project() {
    let dbt_project = DbtProject;
    dbt_project.validate();
}