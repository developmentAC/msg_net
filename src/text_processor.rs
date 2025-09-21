use crate::error::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedText {
    pub original_text: String,
    pub sentences: Vec<String>,
    pub words: Vec<String>,
    pub cleaned_text: String,
    pub metadata: TextMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMetadata {
    pub word_count: usize,
    pub sentence_count: usize,
    pub character_count: usize,
    pub language: String,
    pub source_type: SourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    ChatMessage,
    Document,
    Email,
    Article,
    Unknown,
}

pub struct TextProcessor {
    sentence_regex: Regex,
    word_regex: Regex,
    cleanup_regex: Regex,
    stopwords: HashSet<String>,
    remove_stopwords: bool,
}

impl TextProcessor {
    /// Default English stopwords list - comprehensive list for text analysis
    pub fn default_english_stopwords() -> HashSet<String> {
        let stopwords = [
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he", "in", "is", "it",
            "its", "of", "on", "that", "the", "to", "was", "were", "will", "with", "the", "this", "but",
            "they", "have", "had", "what", "said", "each", "which", "she", "do", "how", "their", "if",
            "up", "out", "many", "then", "them", "these", "so", "some", "her", "would", "make", "like",
            "into", "him", "time", "two", "more", "go", "no", "way", "could", "my", "than", "first",
            "been", "call", "who", "oil", "sit", "now", "find", "down", "day", "did", "get", "may",
            "new", "write", "our", "used", "man", "water", "words", "long", "little", "very", "after",
            "without", "just", "where", "much", "through", "back", "years", "work", "life", "only",
            "over", "know", "should", "came", "any", "here", "take", "why", "help", "put", "different",
            "away", "again", "off", "went", "old", "number", "great", "tell", "men", "say", "small",
            "every", "found", "still", "between", "name", "good", "need", "give", "us", "below", "last",
            "never", "before", "here", "too", "line", "right", "came", "also", "around", "another",
            "came", "three", "high", "part", "small", "end", "why", "asked", "turned", "learn", "point",
            "city", "play", "toward", "five", "himself", "usually", "money", "seen", "didn", "car",
            "morning", "i", "you", "your", "me", "we", "us", "am", "can", "can't", "cannot", "could",
            "couldn't", "did", "didn't", "does", "doesn't", "do", "don't", "had", "hadn't", "has",
            "hasn't", "have", "haven't", "he'd", "he'll", "he's", "here's", "how's", "i'd", "i'll",
            "i'm", "i've", "isn't", "it'd", "it'll", "it's", "let's", "mustn't", "shan't", "she'd",
            "she'll", "she's", "shouldn't", "that's", "there's", "they'd", "they'll", "they're",
            "they've", "we'd", "we're", "we've", "what's", "when's", "where's", "who's", "won't",
            "wouldn't", "you'd", "you'll", "you're", "you've", "about", "above", "across", "after",
            "against", "along", "among", "around", "because", "before", "behind", "below", "beneath",
            "beside", "between", "beyond", "during", "except", "inside", "outside", "through",
            "throughout", "under", "until", "within", "without", "being", "having", "doing"
        ];
        
        stopwords.iter().map(|s| s.to_string()).collect()
    }

    pub fn new() -> Result<Self> {
        Ok(Self {
            sentence_regex: Regex::new(r"[.!?]+\s*")?,
            word_regex: Regex::new(r"\b\w+\b")?,
            cleanup_regex: Regex::new(r"[^\w\s.,!?;:\-\(\)\[\]]")?,
            stopwords: Self::default_english_stopwords(),
            remove_stopwords: true, // Default is to remove stopwords
        })
    }

    pub fn new_with_options(stopwords_file: Option<&str>, remove_stopwords: bool) -> Result<Self> {
        let stopwords = if let Some(file_path) = stopwords_file {
            Self::load_stopwords_from_file(file_path)?
        } else {
            Self::default_english_stopwords()
        };

        Ok(Self {
            sentence_regex: Regex::new(r"[.!?]+\s*")?,
            word_regex: Regex::new(r"\b\w+\b")?,
            cleanup_regex: Regex::new(r"[^\w\s.,!?;:\-\(\)\[\]]")?,
            stopwords,
            remove_stopwords,
        })
    }

    pub fn load_stopwords_from_file(file_path: &str) -> Result<HashSet<String>> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| crate::error::GraphError::Io(e))?;
        
        let stopwords: HashSet<String> = content
            .lines()
            .map(|line| line.trim().to_lowercase())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        
        Ok(stopwords)
    }

    pub fn set_stopwords(&mut self, stopwords: HashSet<String>) {
        self.stopwords = stopwords;
    }

    pub fn set_remove_stopwords(&mut self, remove: bool) {
        self.remove_stopwords = remove;
    }

    pub fn process_text(&self, text: &str, source_type: SourceType) -> Result<ProcessedText> {
        // Print stopword processing status
        if self.remove_stopwords {
            println!("🔍 Processing text with stopword removal enabled");
        } else {
            println!("🔍 Processing text with stopword removal disabled");
        }
        
        let cleaned_text = self.clean_text(text)?;
        let sentences = self.extract_sentences(&cleaned_text)?;
        let words = self.extract_words(&cleaned_text)?;
        
        // Apply stopword removal if enabled
        let filtered_words = if self.remove_stopwords {
            self.remove_stopwords_from_words(&words)
        } else {
            words.clone()
        };

        // Create filtered cleaned text by reconstructing from filtered words
        let filtered_cleaned_text = if self.remove_stopwords {
            self.reconstruct_text_without_stopwords(&cleaned_text)?
        } else {
            cleaned_text.clone()
        };
        
        let metadata = TextMetadata {
            word_count: filtered_words.len(),
            sentence_count: sentences.len(),
            character_count: text.len(),
            language: self.detect_language(&cleaned_text),
            source_type,
        };

        Ok(ProcessedText {
            original_text: text.to_string(),
            sentences,
            words: filtered_words,
            cleaned_text: filtered_cleaned_text,
            metadata,
        })
    }

    fn clean_text(&self, text: &str) -> Result<String> {
        // Remove extra whitespace and normalize
        let text = text.trim();
        let text = text.replace("\t", " ");
        let text = text.replace("\r", "");
        
        // Remove special characters but keep punctuation
        let cleaned = self.cleanup_regex.replace_all(&text, " ");
        
        // Normalize whitespace
        let normalized = Regex::new(r"\s+")?.replace_all(&cleaned, " ");
        
        Ok(normalized.to_string())
    }

    fn extract_sentences(&self, text: &str) -> Result<Vec<String>> {
        let sentences: Vec<String> = self.sentence_regex
            .split(text)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        Ok(sentences)
    }

    fn extract_words(&self, text: &str) -> Result<Vec<String>> {
        let words: Vec<String> = self.word_regex
            .find_iter(text)
            .map(|m| m.as_str().to_lowercase())
            .collect();
        
        Ok(words)
    }

    fn remove_stopwords_from_words(&self, words: &[String]) -> Vec<String> {
        words.iter()
            .filter(|word| !self.stopwords.contains(*word))
            .cloned()
            .collect()
    }

    fn reconstruct_text_without_stopwords(&self, text: &str) -> Result<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let filtered_words: Vec<&str> = words.into_iter()
            .filter(|word| {
                let clean_word = word.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphabetic())
                    .to_string();
                !self.stopwords.contains(&clean_word)
            })
            .collect();
        
        Ok(filtered_words.join(" "))
    }

    fn detect_language(&self, text: &str) -> String {
        // Simple language detection - can be enhanced with proper language detection library
        let common_english_words = ["the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"];
        let word_count = text.split_whitespace().count();
        
        if word_count == 0 {
            return "unknown".to_string();
        }
        
        let english_word_count = text.split_whitespace()
            .filter(|word| common_english_words.contains(&word.to_lowercase().as_str()))
            .count();
        
        let english_ratio = english_word_count as f64 / word_count as f64;
        
        if english_ratio > 0.1 {
            "english".to_string()
        } else {
            "unknown".to_string()
        }
    }

    pub fn extract_context_windows(&self, text: &str, window_size: usize) -> Result<Vec<String>> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut windows = Vec::new();
        
        for i in 0..words.len() {
            let start = if i >= window_size / 2 { i - window_size / 2 } else { 0 };
            let end = std::cmp::min(i + window_size / 2 + 1, words.len());
            
            let window = words[start..end].join(" ");
            windows.push(window);
        }
        
        Ok(windows)
    }

    pub fn extract_key_phrases(&self, text: &str) -> Result<Vec<String>> {
        // Simple key phrase extraction using noun phrases and important terms
        let noun_phrase_regex = Regex::new(r"\b(?:[A-Z][a-z]*\s*){1,3}\b")?;
        let important_term_regex = Regex::new(r"\b(?:important|key|main|primary|essential|critical|vital|crucial)\s+\w+\b")?;
        
        let mut phrases = Vec::new();
        
        // Extract noun phrases
        for cap in noun_phrase_regex.find_iter(text) {
            phrases.push(cap.as_str().trim().to_string());
        }
        
        // Extract important terms
        for cap in important_term_regex.find_iter(text) {
            phrases.push(cap.as_str().trim().to_string());
        }
        
        // Remove duplicates and sort by length (longer phrases first)
        phrases.sort_by(|a, b| b.len().cmp(&a.len()));
        phrases.dedup();
        
        Ok(phrases)
    }
}

impl Default for TextProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to create default TextProcessor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_stopwords_removal() {
        let processor = TextProcessor::new().expect("Failed to create processor");
        let text = "The quick brown fox jumps over the lazy dog and the cat.";
        let processed = processor.process_text(text, SourceType::Document).expect("Failed to process text");
        
        // With stopwords removed, should have fewer words
        assert!(processed.words.len() < text.split_whitespace().count());
        
        // Should not contain common stopwords
        assert!(!processed.words.contains(&"the".to_string()));
        assert!(!processed.words.contains(&"and".to_string()));
        assert!(!processed.words.contains(&"over".to_string()));
        
        // Should contain content words
        assert!(processed.words.contains(&"quick".to_string()));
        assert!(processed.words.contains(&"brown".to_string()));
        assert!(processed.words.contains(&"fox".to_string()));
    }

    #[test]
    fn test_no_stopwords_removal() {
        let processor = TextProcessor::new_with_options(None, false).expect("Failed to create processor");
        let text = "The quick brown fox jumps over the lazy dog.";
        let processed = processor.process_text(text, SourceType::Document).expect("Failed to process text");
        
        // With no stopword removal, should have all words (converted to lowercase)
        let original_word_count = text.split_whitespace().count();
        assert_eq!(processed.words.len(), original_word_count);
        
        // Should contain stopwords
        assert!(processed.words.contains(&"the".to_string()));
        assert!(processed.words.contains(&"over".to_string()));
    }

    #[test]
    fn test_custom_stopwords_file() {
        // Create a temporary stopwords file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, "quick").expect("Failed to write to temp file");
        writeln!(temp_file, "brown").expect("Failed to write to temp file");
        writeln!(temp_file, "# This is a comment").expect("Failed to write to temp file");
        writeln!(temp_file, "jumps").expect("Failed to write to temp file");
        
        let temp_path = temp_file.path().to_str().expect("Failed to get temp path");
        let processor = TextProcessor::new_with_options(Some(temp_path), true).expect("Failed to create processor");
        
        let text = "The quick brown fox jumps over the lazy dog.";
        let processed = processor.process_text(text, SourceType::Document).expect("Failed to process text");
        
        // Should not contain our custom stopwords
        assert!(!processed.words.contains(&"quick".to_string()));
        assert!(!processed.words.contains(&"brown".to_string()));
        assert!(!processed.words.contains(&"jumps".to_string()));
        
        // Should contain other words
        assert!(processed.words.contains(&"the".to_string())); // "the" not in our custom list
        assert!(processed.words.contains(&"fox".to_string()));
        assert!(processed.words.contains(&"dog".to_string()));
    }

    #[test]
    fn test_stopwords_case_insensitive() {
        let processor = TextProcessor::new().expect("Failed to create processor");
        let text = "THE Quick Brown FOX jumps OVER the Lazy DOG.";
        let processed = processor.process_text(text, SourceType::Document).expect("Failed to process text");
        
        // All words should be lowercase and stopwords removed
        assert!(!processed.words.contains(&"the".to_string()));
        assert!(!processed.words.contains(&"over".to_string()));
        assert!(processed.words.contains(&"quick".to_string()));
        assert!(processed.words.contains(&"brown".to_string()));
        assert!(processed.words.contains(&"fox".to_string()));
    }

    #[test]
    fn test_reconstruct_text_without_stopwords() {
        let processor = TextProcessor::new().expect("Failed to create processor");
        let original_text = "The quick brown fox jumps over the lazy dog.";
        
        let reconstructed = processor.reconstruct_text_without_stopwords(original_text)
            .expect("Failed to reconstruct text");
        
        // Should not contain stopwords
        assert!(!reconstructed.contains(" the "));
        assert!(!reconstructed.contains(" over "));
        
        // Should contain content words
        assert!(reconstructed.contains("quick"));
        assert!(reconstructed.contains("brown"));
        assert!(reconstructed.contains("fox"));
    }

    #[test]
    fn test_load_stopwords_from_file() {
        // Create a temporary stopwords file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, "word1").expect("Failed to write to temp file");
        writeln!(temp_file, "word2").expect("Failed to write to temp file");
        writeln!(temp_file, "# comment line").expect("Failed to write to temp file");
        writeln!(temp_file, "word3").expect("Failed to write to temp file");
        writeln!(temp_file, "").expect("Failed to write empty line");
        writeln!(temp_file, "word4").expect("Failed to write to temp file");
        
        let temp_path = temp_file.path().to_str().expect("Failed to get temp path");
        let stopwords = TextProcessor::load_stopwords_from_file(temp_path)
            .expect("Failed to load stopwords");
        
        assert_eq!(stopwords.len(), 4);
        assert!(stopwords.contains("word1"));
        assert!(stopwords.contains("word2"));
        assert!(stopwords.contains("word3"));
        assert!(stopwords.contains("word4"));
        assert!(!stopwords.contains("# comment line"));
        assert!(!stopwords.contains(""));
    }

    #[test]
    fn test_default_english_stopwords() {
        let stopwords = TextProcessor::default_english_stopwords();
        
        // Should contain common English stopwords
        assert!(stopwords.contains("the"));
        assert!(stopwords.contains("and"));
        assert!(stopwords.contains("but"));
        assert!(stopwords.contains("in"));
        assert!(stopwords.contains("on"));
        assert!(stopwords.contains("at"));
        
        // Should be a reasonable size (not too small, not too large)
        assert!(stopwords.len() > 50);
        assert!(stopwords.len() < 500);
    }

    #[test]
    fn test_empty_text_processing() {
        let processor = TextProcessor::new().expect("Failed to create processor");
        let processed = processor.process_text("", SourceType::Document).expect("Failed to process empty text");
        
        assert_eq!(processed.words.len(), 0);
        assert_eq!(processed.sentences.len(), 0);
        assert_eq!(processed.cleaned_text, "");
    }

    #[test]
    fn test_punctuation_handling_with_stopwords() {
        let processor = TextProcessor::new().expect("Failed to create processor");
        let text = "The quick, brown fox! Jumps over the lazy dog?";
        let processed = processor.process_text(text, SourceType::Document).expect("Failed to process text");
        
        // Should handle punctuation correctly while removing stopwords
        assert!(processed.words.contains(&"quick".to_string()));
        assert!(processed.words.contains(&"brown".to_string()));
        assert!(!processed.words.contains(&"the".to_string()));
        assert!(!processed.words.contains(&"over".to_string()));
    }
}
