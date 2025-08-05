use crate::error::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

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
}

impl TextProcessor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sentence_regex: Regex::new(r"[.!?]+\s*")?,
            word_regex: Regex::new(r"\b\w+\b")?,
            cleanup_regex: Regex::new(r"[^\w\s.,!?;:\-\(\)\[\]]")?,
        })
    }

    pub fn process_text(&self, text: &str, source_type: SourceType) -> Result<ProcessedText> {
        let cleaned_text = self.clean_text(text)?;
        let sentences = self.extract_sentences(&cleaned_text)?;
        let words = self.extract_words(&cleaned_text)?;
        
        let metadata = TextMetadata {
            word_count: words.len(),
            sentence_count: sentences.len(),
            character_count: text.len(),
            language: self.detect_language(&cleaned_text),
            source_type,
        };

        Ok(ProcessedText {
            original_text: text.to_string(),
            sentences,
            words,
            cleaned_text,
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
