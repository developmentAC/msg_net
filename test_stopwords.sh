#!/bin/bash

# MSG_NET Test Suite Runner
# This script runs comprehensive tests for the stopword functionality

echo "🧪 MSG_NET Stopword Functionality Test Suite"
echo "=============================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

# Function to run a test and check the result
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    print_status "Running: $test_name"
    if eval "$test_command" >/dev/null 2>&1; then
        print_success "$test_name"
        return 0
    else
        print_error "$test_name"
        return 1
    fi
}

# Function to run cargo test with specific filter
run_cargo_test() {
    local test_name="$1"
    local test_filter="$2"
    
    print_status "Running: $test_name"
    if cargo test "$test_filter" --quiet; then
        print_success "$test_name"
        return 0
    else
        print_error "$test_name"
        return 1
    fi
}

# Ensure we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Please run this script from the msg_net project root directory"
    exit 1
fi

# Track test results
total_tests=0
passed_tests=0

echo "🔧 Building project..."
if ! cargo build --quiet; then
    print_error "Failed to build project"
    exit 1
fi
print_success "Project built successfully"
echo

echo "📋 Running Unit Tests"
echo "--------------------"

# Unit tests
unit_tests=(
    "Default stopwords removal:text_processor::tests::test_default_stopwords_removal"
    "No stopwords removal:text_processor::tests::test_no_stopwords_removal"
    "Custom stopwords file:text_processor::tests::test_custom_stopwords_file"
    "Case insensitive stopwords:text_processor::tests::test_stopwords_case_insensitive"
    "Default English stopwords:text_processor::tests::test_default_english_stopwords"
    "Load stopwords from file:text_processor::tests::test_load_stopwords_from_file"
    "Text reconstruction:text_processor::tests::test_reconstruct_text_without_stopwords"
    "Empty text processing:text_processor::tests::test_empty_text_processing"
    "Punctuation handling:text_processor::tests::test_punctuation_handling_with_stopwords"
)

for test_entry in "${unit_tests[@]}"; do
    IFS=':' read -r test_name test_filter <<< "$test_entry"
    ((total_tests++))
    if run_cargo_test "$test_name" "$test_filter"; then
        ((passed_tests++))
    fi
done

echo
echo "🌐 Running Integration Tests"
echo "----------------------------"

# Integration tests
integration_tests=(
    "Default stopwords behavior:test_default_stopwords_behavior"
    "No remove stopwords flag:test_no_remove_stopwords_flag"
    "Custom stopwords file:test_custom_stopwords_file"
    "Generate with stopwords:test_generate_with_stopwords_options"
    "Generate no remove stopwords:test_generate_no_remove_stopwords"
    "Generate with custom stopwords:test_generate_with_custom_stopwords"
    "Help shows stopwords options:test_help_shows_stopwords_options"
    "Analyze help shows stopwords:test_analyze_help_shows_stopwords_options"
    "Big help includes stopwords:test_big_help_includes_stopwords"
    "Word count comparison:test_word_count_comparison"
    "JSON export with stopwords:test_json_export_with_stopwords"
)

for test_entry in "${integration_tests[@]}"; do
    IFS=':' read -r test_name test_filter <<< "$test_entry"
    ((total_tests++))
    if run_cargo_test "$test_name" "$test_filter"; then
        ((passed_tests++))
    fi
done

echo
echo "⚙️  Running Configuration Tests"
echo "-------------------------------"

# Configuration tests
config_tests=(
    "Config file with stopwords:test_config_file_with_stopwords"
    "Config disable stopwords:test_config_disable_stopwords"
    "CLI overrides config:test_cli_overrides_config"
)

for test_entry in "${config_tests[@]}"; do
    IFS=':' read -r test_name test_filter <<< "$test_entry"
    ((total_tests++))
    if run_cargo_test "$test_name" "$test_filter"; then
        ((passed_tests++))
    fi
done

echo
echo "🎯 Manual Test Examples"
echo "----------------------"
print_status "Creating test files for manual verification..."

# Create test files
cat > test_sample.txt << 'EOF'
The quick brown fox jumps over the lazy dog. The cat is sleeping in the sun. A bird flies through the sky and the wind blows softly.
EOF

cat > custom_stopwords.txt << 'EOF'
quick
brown
fox
cat
bird
EOF

print_success "Test files created: test_sample.txt, custom_stopwords.txt"

echo
print_status "Manual test commands you can run:"
echo "  1. Default stopwords:     cargo run -- analyze -i test_sample.txt --verbose"
echo "  2. No stopwords:          cargo run -- analyze -i test_sample.txt --verbose --no-remove-stopwords"
echo "  3. Custom stopwords:      cargo run -- analyze -i test_sample.txt --verbose --stopwords-file custom_stopwords.txt"
echo "  4. Generate graph:        cargo run -- generate -i test_sample.txt -o test_graph.html"
echo "  5. Generate no stopwords: cargo run -- generate -i test_sample.txt -o test_graph_full.html --no-remove-stopwords"

echo
echo "📊 Test Results Summary"
echo "======================="
echo "Total Tests: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $((total_tests - passed_tests))"

if [ $passed_tests -eq $total_tests ]; then
    print_success "All tests passed! 🎉"
    echo
    print_status "Stopword functionality is working correctly."
    print_status "You can now use the following CLI options:"
    echo "  --stopwords-file <FILE>  : Use custom stopwords from file"
    echo "  --no-remove-stopwords    : Disable stopword removal"
    echo "  (default behavior)       : Use built-in English stopwords"
    exit 0
else
    print_error "Some tests failed. Please check the output above."
    exit 1
fi