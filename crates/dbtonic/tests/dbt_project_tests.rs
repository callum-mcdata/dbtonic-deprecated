use std::fs::File;
use std::io::Write;
use std::env;

use tempfile::tempdir;
use dbtonic::validation::dbt_project_operations::DbtProject;

fn setup_dbt_project(temp_dir: &tempfile::TempDir) {
    let dbt_project_file_path = temp_dir.path().join("dbt_project.yml");
    let mut file = File::create(&dbt_project_file_path).expect("Failed to create dbt_project.yml");
    file.write_all(b"name: test_project\nversion: '1.0.0'\n")
        .expect("Failed to write to dbt_project.yml");
}

#[test]
fn test_validate() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    setup_dbt_project(&temp_dir);
    
    let current_dir = env::current_dir().expect("Failed to get current directory");
    env::set_current_dir(temp_dir.path()).expect("Failed to change to temporary directory");

    let dbt_project = DbtProject;
    dbt_project.validate();
    
    env::set_current_dir(current_dir).expect("Failed to change back to original directory");
}
