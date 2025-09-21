use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;
use serde_json::json;

const TEST_TEXT: &str = "The quick brown fox jumps over the lazy dog. The cat is sleeping in the sun.";

#[test]
fn test_config_file_with_stopwords() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    let config_file = temp_dir.path().join("config.json");
    let output_file = temp_dir.path().join("output.html");
    
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");
    
    // Create config with stopword settings
    let config = json!({
        "text_processing": {
            "remove_stopwords": true,
            "custom_stopwords": ["quick", "brown", "fox"]
        },
        "node_colors": {
            "entity": "#FF6B6B",
            "relationship": "#4ECDC4",
            "concept": "#45B7D1",
            "attribute": "#FFA07A"
        },
        "node_shapes": {
            "entity": "ellipse",
            "relationship": "box",
            "concept": "circle",
            "attribute": "diamond"
        },
        "layout": {
            "algorithm": "hierarchical",
            "spacing": 200.0,
            "hierarchical": true
        },
        "physics": {
            "enabled": true,
            "stabilization": true,
            "repulsion": 200.0,
            "spring_length": 150.0,
            "spring_constant": 0.04
        },
        "extraction": {
            "use_llm": false,
            "llm_model": "llama3.2",
            "llm_endpoint": "http://localhost:11434/api/generate",
            "entity_patterns": [
                "\\b[A-Z][a-z]+(?:\\s+[A-Z][a-z]+)*\\b",
                "\\b(?:person|people|individual|user|customer|client)\\b"
            ],
            "relationship_patterns": [
                "\\b(?:has|have|is|are|was|were|contains|includes|owns|belongs)\\b",
                "\\b(?:connected to|related to|associated with|linked to)\\b"
            ],
            "concept_patterns": [
                "\\b(?:concept|idea|principle|theory|method|approach|strategy)\\b",
                "\\b(?:system|process|workflow|procedure|protocol)\\b"
            ]
        }
    });
    
    fs::write(&config_file, config.to_string()).expect("Failed to write config file");

    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("generate")
        .arg("-i")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .arg("-c")
        .arg(&config_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Graph exported successfully"));
}

#[test]
fn test_config_disable_stopwords() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    let config_file = temp_dir.path().join("config.json");
    
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");
    
    // Create config with stopwords disabled
    let config = json!({
        "node_colors": {
            "entity": "#FF6B6B",
            "relationship": "#4ECDC4",
            "concept": "#45B7D1",
            "attribute": "#FFA07A"
        },
        "node_shapes": {
            "entity": "ellipse",
            "relationship": "box",
            "concept": "circle",
            "attribute": "diamond"
        },
        "layout": {
            "algorithm": "hierarchical",
            "spacing": 200.0,
            "hierarchical": true
        },
        "physics": {
            "enabled": true,
            "stabilization": true,
            "repulsion": 200.0,
            "spring_length": 150.0,
            "spring_constant": 0.04
        },
        "text_processing": {
            "remove_stopwords": false
        },
        "extraction": {
            "use_llm": false,
            "llm_model": "llama3.2",
            "llm_endpoint": "http://localhost:11434/api/generate",
            "entity_patterns": [
                "\\b[A-Z][a-z]+(?:\\s+[A-Z][a-z]+)*\\b"
            ],
            "relationship_patterns": [
                "\\b(?:has|have|is|are|was|were|contains|includes|owns|belongs)\\b"
            ],
            "concept_patterns": [
                "\\b(?:concept|idea|principle|theory|method|approach|strategy)\\b"
            ]
        }
    });
    
    fs::write(&config_file, config.to_string()).expect("Failed to write config file");

    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("analyze")
        .arg("-i")
        .arg(&input_file)
        .arg("-c")
        .arg(&config_file)
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Word count:"));
}

#[test]
fn test_cli_overrides_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    let config_file = temp_dir.path().join("config.json");
    
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");
    
    // Create config with stopwords enabled
    let config = json!({
        "node_colors": {
            "entity": "#FF6B6B",
            "relationship": "#4ECDC4",
            "concept": "#45B7D1",
            "attribute": "#FFA07A"
        },
        "node_shapes": {
            "entity": "ellipse",
            "relationship": "box",
            "concept": "circle",
            "attribute": "diamond"
        },
        "layout": {
            "algorithm": "hierarchical",
            "spacing": 200.0,
            "hierarchical": true
        },
        "physics": {
            "enabled": true,
            "stabilization": true,
            "repulsion": 200.0,
            "spring_length": 150.0,
            "spring_constant": 0.04
        },
        "text_processing": {
            "remove_stopwords": true
        },
        "extraction": {
            "use_llm": false,
            "llm_model": "llama3.2",
            "llm_endpoint": "http://localhost:11434/api/generate",
            "entity_patterns": [
                "\\b[A-Z][a-z]+(?:\\s+[A-Z][a-z]+)*\\b"
            ],
            "relationship_patterns": [
                "\\b(?:has|have|is|are|was|were|contains|includes|owns|belongs)\\b"
            ],
            "concept_patterns": [
                "\\b(?:concept|idea|principle|theory|method|approach|strategy)\\b"
            ]
        }
    });
    
    fs::write(&config_file, config.to_string()).expect("Failed to write config file");

    // CLI should override config setting
    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("analyze")
        .arg("-i")
        .arg(&input_file)
        .arg("-c")
        .arg(&config_file)
        .arg("--no-remove-stopwords")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Word count:"));
}