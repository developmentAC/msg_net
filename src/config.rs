use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub node_colors: NodeColors,
    pub node_shapes: NodeShapes,
    pub layout: LayoutConfig,
    pub physics: PhysicsConfig,
    pub extraction: ExtractionConfig,
    pub text_processing: TextProcessingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeColors {
    pub entity: String,
    pub relationship: String,
    pub concept: String,
    pub attribute: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeShapes {
    pub entity: String,
    pub relationship: String,
    pub concept: String,
    pub attribute: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub algorithm: String,
    pub spacing: f64,
    pub hierarchical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub enabled: bool,
    pub stabilization: bool,
    pub repulsion: f64,
    pub spring_length: f64,
    pub spring_constant: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    pub use_llm: bool,
    pub llm_model: String,
    pub llm_endpoint: String,
    pub entity_patterns: Vec<String>,
    pub relationship_patterns: Vec<String>,
    pub concept_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProcessingConfig {
    pub remove_stopwords: bool,
    pub stopwords_file: Option<String>,
    pub custom_stopwords: Option<Vec<String>>,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            node_colors: NodeColors {
                entity: "#FF6B6B".to_string(),
                relationship: "#4ECDC4".to_string(),
                concept: "#45B7D1".to_string(),
                attribute: "#FFA07A".to_string(),
            },
            node_shapes: NodeShapes {
                entity: "ellipse".to_string(),
                relationship: "box".to_string(),
                concept: "circle".to_string(),
                attribute: "diamond".to_string(),
            },
            layout: LayoutConfig {
                algorithm: "hierarchical".to_string(),
                spacing: 200.0,
                hierarchical: true,
            },
            physics: PhysicsConfig {
                enabled: true,
                stabilization: true,
                repulsion: 200.0,
                spring_length: 150.0,
                spring_constant: 0.04,
            },
            extraction: ExtractionConfig::default(),
            text_processing: TextProcessingConfig::default(),
        }
    }
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            use_llm: false,
            llm_model: "llama3.2".to_string(),
            llm_endpoint: "http://localhost:11434/api/generate".to_string(),
            entity_patterns: vec![
                r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b".to_string(),
                r"\b(?:person|people|individual|user|customer|client)\b".to_string(),
            ],
            relationship_patterns: vec![
                r"\b(?:has|have|is|are|was|were|contains|includes|owns|belongs)\b".to_string(),
                r"\b(?:connected to|related to|associated with|linked to)\b".to_string(),
            ],
            concept_patterns: vec![
                r"\b(?:concept|idea|principle|theory|method|approach|strategy)\b".to_string(),
                r"\b(?:system|process|workflow|procedure|protocol)\b".to_string(),
            ],
        }
    }
}

impl Default for TextProcessingConfig {
    fn default() -> Self {
        Self {
            remove_stopwords: true,
            stopwords_file: None,
            custom_stopwords: None,
        }
    }
}
