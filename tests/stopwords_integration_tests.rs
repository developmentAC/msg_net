use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

const TEST_TEXT: &str = "The quick brown fox jumps over the lazy dog. The cat is sleeping in the sun. A bird flies through the sky.";

#[test]
fn test_default_stopwords_behavior() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");

    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("analyze")
        .arg("-i")
        .arg(&input_file)
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Word count:"))
        .stdout(predicate::str::contains("Sentence count: 3"));
}

#[test]
fn test_no_remove_stopwords_flag() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");

    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("analyze")
        .arg("-i")
        .arg(&input_file)
        .arg("--verbose")
        .arg("--no-remove-stopwords");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Word count:"));
        
    // Test that word count is higher when stopwords are not removed
    // We'll run both commands and compare in a separate test
}

#[test]
fn test_custom_stopwords_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    let stopwords_file = temp_dir.path().join("stopwords.txt");
    
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");
    fs::write(&stopwords_file, "quick\nbrown\nfox\ncat\nbird").expect("Failed to write stopwords file");

    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("analyze")
        .arg("-i")
        .arg(&input_file)
        .arg("--verbose")
        .arg("--stopwords-file")
        .arg(&stopwords_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Word count:"));
}

#[test]
fn test_generate_with_stopwords_options() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    let output_file = temp_dir.path().join("output.html");
    
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");

    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("generate")
        .arg("-i")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Graph exported successfully"));
}

#[test]
fn test_generate_no_remove_stopwords() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    let output_file = temp_dir.path().join("output.html");
    
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");

    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("generate")
        .arg("-i")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .arg("--no-remove-stopwords");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Graph exported successfully"));
}

#[test]
fn test_generate_with_custom_stopwords() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    let output_file = temp_dir.path().join("output.html");
    let stopwords_file = temp_dir.path().join("stopwords.txt");
    
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");
    fs::write(&stopwords_file, "quick\nbrown\nfox").expect("Failed to write stopwords file");

    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("generate")
        .arg("-i")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .arg("--stopwords-file")
        .arg(&stopwords_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Graph exported successfully"));
}

#[test]
fn test_invalid_stopwords_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    let output_file = temp_dir.path().join("output.html");
    let nonexistent_file = temp_dir.path().join("nonexistent.txt");
    
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");

    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("generate")
        .arg("-i")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .arg("--stopwords-file")
        .arg(&nonexistent_file);

    cmd.assert()
        .failure();
}

#[test]
fn test_help_shows_stopwords_options() {
    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("generate").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("stopwords-file"))
        .stdout(predicate::str::contains("no-remove-stopwords"));
}

#[test]
fn test_analyze_help_shows_stopwords_options() {
    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("analyze").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("stopwords-file"))
        .stdout(predicate::str::contains("no-remove-stopwords"));
}

#[test]
fn test_big_help_includes_stopwords() {
    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("big-help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("STOPWORD PROCESSING"))
        .stdout(predicate::str::contains("no-remove-stopwords"))
        .stdout(predicate::str::contains("stopwords-file"));
}

#[test]
fn test_word_count_comparison() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");

    // Test with stopwords removed (default)
    let mut cmd1 = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd1.arg("analyze")
        .arg("-i")
        .arg(&input_file)
        .arg("--verbose");

    let output1 = cmd1.assert().success().get_output().stdout.clone();
    let output1_str = String::from_utf8(output1).expect("Invalid UTF-8");

    // Test without stopwords removed
    let mut cmd2 = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd2.arg("analyze")
        .arg("-i")
        .arg(&input_file)
        .arg("--verbose")
        .arg("--no-remove-stopwords");

    let output2 = cmd2.assert().success().get_output().stdout.clone();
    let output2_str = String::from_utf8(output2).expect("Invalid UTF-8");

    // Extract word counts and verify that no-stopwords has more words
    let extract_word_count = |output: &str| -> Option<usize> {
        for line in output.lines() {
            if line.contains("Word count:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(count_str) = parts.get(2) {
                    return count_str.parse().ok();
                }
            }
        }
        None
    };

    let count_with_stopwords_removed = extract_word_count(&output1_str)
        .expect("Could not extract word count from first output");
    let count_without_stopwords_removed = extract_word_count(&output2_str)
        .expect("Could not extract word count from second output");

    // Without stopwords removed should have more words
    assert!(count_without_stopwords_removed > count_with_stopwords_removed,
        "Expected {} > {} (words without stopword removal should be more than with stopword removal)",
        count_without_stopwords_removed, count_with_stopwords_removed);
}

#[test]
fn test_json_export_with_stopwords() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");
    let output_file = temp_dir.path().join("output.json");
    
    fs::write(&input_file, TEST_TEXT).expect("Failed to write test file");

    let mut cmd = Command::cargo_bin("msg_net").expect("Failed to find binary");
    cmd.arg("generate")
        .arg("-i")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .arg("-f")
        .arg("json")
        .arg("--include-metadata")
        .current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Graph exported successfully"));

    // Check if file was created in 0_networks directory (application creates this in current working directory)
    let output_in_networks = temp_dir.path().join("0_networks").join("output.json");
    assert!(output_in_networks.exists(), 
            "Expected output file at {:?}", output_in_networks);
}