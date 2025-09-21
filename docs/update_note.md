# Update notes

## 21 Sept 2025

### Default stopword processing

* `cargo run -- generate -i document.txt -o clean_graph.html`

### Disable stopwords

* `cargo run -- generate -i document.txt -o full_graph.html --no-remove-stopwords`

### Custom stopwords

* `echo -e 'the\na\nand\nor' > custom_stopwords.txt`
* `cargo run -- generate -i document.txt -o custom_graph.html --stopwords-file custom_stopwords.txt`

### Compare processing

* `cargo run -- analyze -i document.txt --verbose`
* `cargo run -- analyze -i document.txt --verbose --no-remove-stopwords`

### Deep analysis with stopwords

* `cargo run -- generate -i document.txt -o deep_custom.html --use-llm --deep-analysis --stopwords-file custom.txt`

## Testing Commands

### Run All Tests

* `cargo test` - Run all unit and integration tests
* `cargo test -- --nocapture` - Run tests with detailed output
* `./test_stopwords.sh` - Run comprehensive test suite with colored output

### Stopword-Specific Tests

* `cargo test text_processor::tests::stopwords` - Run stopword unit tests
* `cargo test --test stopwords_integration_tests` - Run stopword integration tests
* `cargo test --test config_stopwords_tests` - Run configuration-based stopword tests

### Test Categories

* **Unit Tests (9 tests)**: Core stopword functionality, default lists, custom file loading
* **Integration Tests (12 tests)**: CLI argument handling, file processing, help documentation  
* **Configuration Tests (3 tests)**: JSON configuration with stopword settings

### Manual Testing Examples

* Create sample stopwords file: `echo -e 'test\nsample\nexample' > test_stopwords.txt`
* Test with custom stopwords: `cargo run -- generate -i sampleData/sample.txt -o test_output.html --stopwords-file test_stopwords.txt`
* Compare results: `cargo run -- analyze -i sampleData/sample.txt --verbose --stopwords-file test_stopwords.txt`
