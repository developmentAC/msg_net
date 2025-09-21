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

