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
