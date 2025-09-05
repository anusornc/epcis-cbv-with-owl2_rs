use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;
use std::fs;
use std::path::Path;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("EPCIS Knowledge Graph demo"))
        .stdout(contains("Commands:"))
        .stdout(contains("serve"))
        .stdout(contains("load"))
        .stdout(contains("query"))
        .stdout(contains("validate"))
        .stdout(contains("init"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(contains("epcis-knowledge-graph"));
}

#[test]
fn test_cli_verbose_flag() {
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args(["--verbose", "init", "--help"])
        .assert()
        .success();
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args(["init", "--db-path", &db_path.to_string_lossy()])
        .assert()
        .success()
        .stdout(contains("Knowledge graph initialization not yet implemented"));
}

#[test]
fn test_init_command_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args(["init", "--db-path", &db_path.to_string_lossy(), "--force"])
        .assert()
        .success()
        .stdout(contains("Knowledge graph initialization not yet implemented"));
}

#[test]
fn test_serve_command() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args(["serve", "--db-path", &db_path.to_string_lossy(), "--port", "8081"])
        .assert()
        .success()
        .stdout(contains("Server functionality not yet implemented"));
}

#[test]
fn test_load_command() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    // Create a dummy ontology file
    let ontology_file = temp_dir.path().join("test.ttl");
    fs::write(&ontology_file, "@prefix ex: <http://example.com/> .").unwrap();
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args([
        "load", 
        "--db-path", &db_path.to_string_lossy(),
        &ontology_file.to_string_lossy()
    ])
    .assert()
    .success()
    .stdout(contains("Ontology loading not yet implemented"));
}

#[test]
fn test_load_command_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    // Create dummy ontology files
    let file1 = temp_dir.path().join("test1.ttl");
    let file2 = temp_dir.path().join("test2.ttl");
    fs::write(&file1, "@prefix ex1: <http://example.com/1/> .").unwrap();
    fs::write(&file2, "@prefix ex2: <http://example.com/2/> .").unwrap();
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args([
        "load", 
        "--db-path", &db_path.to_string_lossy(),
        &file1.to_string_lossy(),
        &file2.to_string_lossy()
    ])
    .assert()
    .success()
    .stdout(contains("Ontology loading not yet implemented"));
}

#[test]
fn test_load_command_missing_files() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args(["load", "--db-path", &db_path.to_string_lossy()])
        .assert()
        .failure()
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn test_query_command() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let query = "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10";
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args([
        "query", 
        "--db-path", &db_path.to_string_lossy(),
        query,
        "--format", "json"
    ])
    .assert()
    .success()
    .stdout(contains("Query execution not yet implemented"));
}

#[test]
fn test_query_command_missing_query() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args(["query", "--db-path", &db_path.to_string_lossy()])
        .assert()
        .failure()
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn test_validate_command() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    // Create a dummy event file
    let event_file = temp_dir.path().join("event.json");
    let event_data = r#"
    {
        "eventID": "test-event-001",
        "eventType": "ObjectEvent",
        "eventTime": "2024-01-01T00:00:00Z",
        "recordTime": "2024-01-01T00:00:00Z",
        "eventAction": "ADD",
        "epcList": ["urn:epc:id:sgtin:123456.789.100"],
        "bizStep": "commissioning",
        "disposition": "active"
    }
    "#;
    fs::write(&event_file, event_data).unwrap();
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args([
        "validate", 
        "--db-path", &db_path.to_string_lossy(),
        &event_file.to_string_lossy()
    ])
    .assert()
    .success()
    .stdout(contains("Event validation not yet implemented"));
}

#[test]
fn test_validate_command_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args(["validate", "--db-path", &db_path.to_string_lossy()])
        .assert()
        .failure()
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn test_config_file_option() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("test_config.toml");
    
    // Create a test config file
    let config_content = r#"
    [database]
    path = "./test_data"
    
    [server]
    port = 9000
    
    [logging]
    level = "debug"
    "#;
    fs::write(&config_file, config_content).unwrap();
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args(["--config", &config_file.to_string_lossy(), "--help"])
        .assert()
        .success();
}

#[test]
fn test_unknown_subcommand() {
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args(["unknown-command"])
        .assert()
        .failure()
        .stderr(contains("unrecognized subcommand"));
}

#[test]
fn test_invalid_port() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let mut cmd = Command::cargo_bin("epcis-knowledge-graph").unwrap();
    cmd.args(["serve", "--db-path", &db_path.to_string_lossy(), "--port", "99999"])
        .assert()
        .failure()
        .stderr(contains("is not in 0..=65535"));
}