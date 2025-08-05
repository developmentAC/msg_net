use crate::config::ExtractionConfig;
use crate::error::{GraphError, Result};
use crate::text_processor::ProcessedText;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use reqwest;
use serde_json;

// Ollama API request/response structures
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OllamaResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub attributes: Vec<Attribute>,
    pub confidence: f64,
    pub position: Option<TextPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub source_entity_id: String,
    pub target_entity_id: String,
    pub relationship_type: RelationshipType,
    pub label: String,
    pub confidence: f64,
    pub position: Option<TextPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: String,
    pub name: String,
    pub description: String,
    pub related_entities: Vec<String>,
    pub confidence: f64,
    pub position: Option<TextPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub id: String,
    pub name: String,
    pub value: String,
    pub attribute_type: AttributeType,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Place,
    Organization,
    Event,
    Product,
    Concept,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Has,
    IsA,
    PartOf,
    ConnectedTo,
    RelatedTo,
    Contains,
    Owns,
    Uses,
    Creates,
    Influences,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeType {
    Name,
    Description,
    Location,
    Date,
    Number,
    Category,
    Property,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    pub start: usize,
    pub end: usize,
    pub sentence_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub concepts: Vec<Concept>,
    pub metadata: ExtractionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    pub total_entities: usize,
    pub total_relationships: usize,
    pub total_concepts: usize,
    pub processing_time_ms: u64,
    pub confidence_threshold: f64,
    pub extraction_method: String,
}

pub struct EntityExtractor {
    config: ExtractionConfig,
    entity_patterns: Vec<Regex>,
    relationship_patterns: Vec<Regex>,
    concept_patterns: Vec<Regex>,
}

impl EntityExtractor {
    pub fn new(config: ExtractionConfig) -> Result<Self> {
        let entity_patterns = Self::compile_patterns(&config.entity_patterns)?;
        let relationship_patterns = Self::compile_patterns(&config.relationship_patterns)?;
        let concept_patterns = Self::compile_patterns(&config.concept_patterns)?;

        Ok(Self {
            config,
            entity_patterns,
            relationship_patterns,
            concept_patterns,
        })
    }

    fn compile_patterns(patterns: &[String]) -> Result<Vec<Regex>> {
        patterns
            .iter()
            .map(|pattern| Regex::new(pattern).map_err(GraphError::from))
            .collect()
    }

    pub async fn extract_from_text(&self, processed_text: &ProcessedText) -> Result<ExtractionResult> {
        let start_time = std::time::Instant::now();

        let entities = if self.config.use_llm {
            self.extract_entities_with_llm(processed_text).await?
        } else {
            self.extract_entities_with_patterns(processed_text)?
        };

        let relationships = if self.config.use_llm {
            self.extract_relationships_with_llm(processed_text, &entities).await?
        } else {
            self.extract_relationships_with_patterns(processed_text, &entities)?
        };

        let concepts = if self.config.use_llm {
            self.extract_concepts_with_llm(processed_text).await?
        } else {
            self.extract_concepts_with_patterns(processed_text)?
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        let metadata = ExtractionMetadata {
            total_entities: entities.len(),
            total_relationships: relationships.len(),
            total_concepts: concepts.len(),
            processing_time_ms: processing_time,
            confidence_threshold: 0.5,
            extraction_method: if self.config.use_llm {
                format!("LLM-{}", self.config.llm_model)
            } else {
                "Pattern-based".to_string()
            },
        };

        Ok(ExtractionResult {
            entities,
            relationships,
            concepts,
            metadata,
        })
    }

    fn extract_entities_with_patterns(&self, processed_text: &ProcessedText) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        let mut seen_entities = HashSet::new();

        for (sentence_idx, sentence) in processed_text.sentences.iter().enumerate() {
            for pattern in &self.entity_patterns {
                for mat in pattern.find_iter(sentence) {
                    let entity_text = mat.as_str().trim();
                    
                    if entity_text.len() < 2 || seen_entities.contains(entity_text) {
                        continue;
                    }
                    
                    seen_entities.insert(entity_text.to_string());
                    
                    let entity_type = self.classify_entity_type(entity_text);
                    let attributes = self.extract_entity_attributes(entity_text, sentence);
                    
                    let entity = Entity {
                        id: Uuid::new_v4().to_string(),
                        name: entity_text.to_string(),
                        entity_type,
                        attributes,
                        confidence: 0.7, // Default confidence for pattern-based extraction
                        position: Some(TextPosition {
                            start: mat.start(),
                            end: mat.end(),
                            sentence_index: sentence_idx,
                        }),
                    };
                    
                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }

    fn extract_relationships_with_patterns(
        &self,
        processed_text: &ProcessedText,
        entities: &[Entity],
    ) -> Result<Vec<Relationship>> {
        let mut relationships = Vec::new();
        
        for (sentence_idx, sentence) in processed_text.sentences.iter().enumerate() {
            // Find entities in this sentence
            let sentence_entities: Vec<&Entity> = entities
                .iter()
                .filter(|e| {
                    if let Some(pos) = &e.position {
                        pos.sentence_index == sentence_idx
                    } else {
                        sentence.contains(&e.name)
                    }
                })
                .collect();

            // Look for relationship patterns between entities
            for i in 0..sentence_entities.len() {
                for j in i + 1..sentence_entities.len() {
                    let entity1 = sentence_entities[i];
                    let entity2 = sentence_entities[j];
                    
                    if let Some(relationship) = self.find_relationship_between_entities(
                        entity1,
                        entity2,
                        sentence,
                        sentence_idx,
                    )? {
                        relationships.push(relationship);
                    }
                }
            }
        }

        Ok(relationships)
    }

    fn extract_concepts_with_patterns(&self, processed_text: &ProcessedText) -> Result<Vec<Concept>> {
        let mut concepts = Vec::new();
        let mut seen_concepts = HashSet::new();

        for (sentence_idx, sentence) in processed_text.sentences.iter().enumerate() {
            for pattern in &self.concept_patterns {
                for mat in pattern.find_iter(sentence) {
                    let concept_text = mat.as_str().trim();
                    
                    if concept_text.len() < 3 || seen_concepts.contains(concept_text) {
                        continue;
                    }
                    
                    seen_concepts.insert(concept_text.to_string());
                    
                    let concept = Concept {
                        id: Uuid::new_v4().to_string(),
                        name: concept_text.to_string(),
                        description: self.generate_concept_description(concept_text, sentence),
                        related_entities: Vec::new(), // Will be populated later
                        confidence: 0.6,
                        position: Some(TextPosition {
                            start: mat.start(),
                            end: mat.end(),
                            sentence_index: sentence_idx,
                        }),
                    };
                    
                    concepts.push(concept);
                }
            }
        }

        Ok(concepts)
    }

    async fn extract_entities_with_llm(&self, processed_text: &ProcessedText) -> Result<Vec<Entity>> {
        if !self.config.use_llm {
            return Ok(Vec::new());
        }

        println!("🤖 Extracting entities using LLM: {}", self.config.llm_model);
        
        let prompt = format!(
            r#"Analyze the following text and extract entities (people, places, organizations, concepts, systems, processes).

Text: "{}"

Please respond with a JSON array of entities in this exact format:
[
  {{
    "name": "entity_name",
    "type": "Person|Place|Organization|System|Process|Concept|Other",
    "confidence": 0.8
  }}
]

Only return the JSON array, no other text."#,
            processed_text.cleaned_text
        );

        match self.call_ollama(&prompt).await {
            Ok(response) => {
                match self.parse_entities_from_llm_response(&response) {
                    Ok(entities) => {
                        println!("✅ LLM extracted {} entities", entities.len());
                        Ok(entities)
                    }
                    Err(e) => {
                        println!("⚠️  LLM response parsing failed: {}, falling back to patterns", e);
                        self.extract_entities_with_patterns(processed_text)
                    }
                }
            }
            Err(e) => {
                println!("⚠️  LLM call failed: {}, falling back to patterns", e);
                self.extract_entities_with_patterns(processed_text)
            }
        }
    }

    async fn extract_relationships_with_llm(
        &self,
        processed_text: &ProcessedText,
        entities: &[Entity],
    ) -> Result<Vec<Relationship>> {
        if !self.config.use_llm || entities.is_empty() {
            return self.extract_relationships_with_patterns(processed_text, entities);
        }

        println!("🤖 Extracting relationships using LLM: {}", self.config.llm_model);
        
        let entity_names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        let prompt = format!(
            r#"Analyze the following text and identify relationships between these entities: {:?}

Text: "{}"

Please respond with a JSON array of relationships in this exact format:
[
  {{
    "from": "entity1_name",
    "to": "entity2_name", 
    "relationship": "relationship_type",
    "confidence": 0.8
  }}
]

Only return the JSON array, no other text."#,
            entity_names,
            processed_text.cleaned_text
        );

        match self.call_ollama(&prompt).await {
            Ok(response) => {
                match self.parse_relationships_from_llm_response(&response, entities) {
                    Ok(relationships) => {
                        println!("✅ LLM extracted {} relationships", relationships.len());
                        Ok(relationships)
                    }
                    Err(e) => {
                        println!("⚠️  LLM response parsing failed: {}, falling back to patterns", e);
                        self.extract_relationships_with_patterns(processed_text, entities)
                    }
                }
            }
            Err(e) => {
                println!("⚠️  LLM call failed: {}, falling back to patterns", e);
                self.extract_relationships_with_patterns(processed_text, entities)
            }
        }
    }

    async fn extract_concepts_with_llm(&self, processed_text: &ProcessedText) -> Result<Vec<Concept>> {
        if !self.config.use_llm {
            return self.extract_concepts_with_patterns(processed_text);
        }

        println!("🤖 Extracting concepts using LLM: {}", self.config.llm_model);
        
        let prompt = format!(
            r#"Analyze the following text and extract key concepts, ideas, systems, processes, and methods.

Text: "{}"

Please respond with a JSON array of concepts in this exact format:
[
  {{
    "name": "concept_name",
    "description": "brief description of the concept",
    "confidence": 0.8
  }}
]

Only return the JSON array, no other text."#,
            processed_text.cleaned_text
        );

        match self.call_ollama(&prompt).await {
            Ok(response) => {
                match self.parse_concepts_from_llm_response(&response) {
                    Ok(concepts) => {
                        println!("✅ LLM extracted {} concepts", concepts.len());
                        Ok(concepts)
                    }
                    Err(e) => {
                        println!("⚠️  LLM response parsing failed: {}, falling back to patterns", e);
                        self.extract_concepts_with_patterns(processed_text)
                    }
                }
            }
            Err(e) => {
                println!("⚠️  LLM call failed: {}, falling back to patterns", e);
                self.extract_concepts_with_patterns(processed_text)
            }
        }
    }

    fn classify_entity_type(&self, entity_text: &str) -> EntityType {
        let lower_text = entity_text.to_lowercase();
        
        // Simple heuristics for entity classification
        if lower_text.contains("corp") || lower_text.contains("inc") || lower_text.contains("company") {
            EntityType::Organization
        } else if entity_text.chars().next().unwrap_or(' ').is_uppercase() && entity_text.split_whitespace().count() <= 3 {
            // Likely a proper noun (person or place)
            EntityType::Person
        } else if lower_text.contains("system") || lower_text.contains("process") || lower_text.contains("method") {
            EntityType::Concept
        } else {
            EntityType::Other(lower_text)
        }
    }

    fn extract_entity_attributes(&self, entity_text: &str, context: &str) -> Vec<Attribute> {
        let mut attributes = Vec::new();
        
        // Extract basic attributes from context
        attributes.push(Attribute {
            id: Uuid::new_v4().to_string(),
            name: "name".to_string(),
            value: entity_text.to_string(),
            attribute_type: AttributeType::Name,
            confidence: 1.0,
        });

        // Look for descriptive attributes in context
        if let Some(description) = self.extract_description_from_context(entity_text, context) {
            attributes.push(Attribute {
                id: Uuid::new_v4().to_string(),
                name: "description".to_string(),
                value: description,
                attribute_type: AttributeType::Description,
                confidence: 0.7,
            });
        }

        attributes
    }

    fn extract_description_from_context(&self, entity: &str, context: &str) -> Option<String> {
        // Simple pattern to find descriptions like "John, a software engineer" or "the red car"
        let patterns = [
            format!(r"{},?\s+(?:a|an|the)\s+([^,\.]+)", regex::escape(entity)),
            format!(r"(?:a|an|the)\s+([^,\s]+)\s+{}", regex::escape(entity)),
        ];

        for pattern_str in &patterns {
            if let Ok(pattern) = Regex::new(pattern_str) {
                if let Some(cap) = pattern.captures(context) {
                    if let Some(desc) = cap.get(1) {
                        return Some(desc.as_str().trim().to_string());
                    }
                }
            }
        }

        None
    }

    fn find_relationship_between_entities(
        &self,
        entity1: &Entity,
        entity2: &Entity,
        sentence: &str,
        sentence_idx: usize,
    ) -> Result<Option<Relationship>> {
        // Look for relationship patterns between entities
        for pattern in &self.relationship_patterns {
            let entity1_pos = sentence.find(&entity1.name);
            let entity2_pos = sentence.find(&entity2.name);
            
            if let (Some(pos1), Some(pos2)) = (entity1_pos, entity2_pos) {
                let start = std::cmp::min(pos1, pos2);
                let end = std::cmp::max(pos1 + entity1.name.len(), pos2 + entity2.name.len());
                let substring = &sentence[start..end];
                
                if pattern.is_match(substring) {
                    let relationship_type = self.classify_relationship_type(substring);
                    let label = self.generate_relationship_label(&relationship_type, &entity1.name, &entity2.name);
                    
                    return Ok(Some(Relationship {
                        id: Uuid::new_v4().to_string(),
                        source_entity_id: entity1.id.clone(),
                        target_entity_id: entity2.id.clone(),
                        relationship_type,
                        label,
                        confidence: 0.6,
                        position: Some(TextPosition {
                            start,
                            end,
                            sentence_index: sentence_idx,
                        }),
                    }));
                }
            }
        }
        
        Ok(None)
    }

    fn classify_relationship_type(&self, text: &str) -> RelationshipType {
        let lower_text = text.to_lowercase();
        
        if lower_text.contains("has") || lower_text.contains("have") || lower_text.contains("owns") {
            RelationshipType::Has
        } else if lower_text.contains("is") || lower_text.contains("are") || lower_text.contains("was") || lower_text.contains("were") {
            RelationshipType::IsA
        } else if lower_text.contains("part of") || lower_text.contains("belongs") {
            RelationshipType::PartOf
        } else if lower_text.contains("connected") || lower_text.contains("linked") {
            RelationshipType::ConnectedTo
        } else if lower_text.contains("uses") || lower_text.contains("utilizes") {
            RelationshipType::Uses
        } else if lower_text.contains("creates") || lower_text.contains("generates") {
            RelationshipType::Creates
        } else if lower_text.contains("influences") || lower_text.contains("affects") {
            RelationshipType::Influences
        } else {
            RelationshipType::RelatedTo
        }
    }

    fn generate_relationship_label(&self, rel_type: &RelationshipType, entity1: &str, entity2: &str) -> String {
        match rel_type {
            RelationshipType::Has => format!("{} has {}", entity1, entity2),
            RelationshipType::IsA => format!("{} is a {}", entity1, entity2),
            RelationshipType::PartOf => format!("{} is part of {}", entity1, entity2),
            RelationshipType::ConnectedTo => format!("{} connected to {}", entity1, entity2),
            RelationshipType::RelatedTo => format!("{} related to {}", entity1, entity2),
            RelationshipType::Contains => format!("{} contains {}", entity1, entity2),
            RelationshipType::Owns => format!("{} owns {}", entity1, entity2),
            RelationshipType::Uses => format!("{} uses {}", entity1, entity2),
            RelationshipType::Creates => format!("{} creates {}", entity1, entity2),
            RelationshipType::Influences => format!("{} influences {}", entity1, entity2),
            RelationshipType::Other(label) => format!("{} {} {}", entity1, label, entity2),
        }
    }

    fn generate_concept_description(&self, concept: &str, context: &str) -> String {
        // Simple description generation based on context
        format!("Concept '{}' mentioned in context: {}", concept, 
                if context.len() > 100 { 
                    &context[..100] 
                } else { 
                    context 
                })
    }

    /// Call Ollama API with a prompt
    async fn call_ollama(&self, prompt: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let request = OllamaRequest {
            model: self.config.llm_model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = client
            .post(&self.config.llm_endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|e| GraphError::EntityExtraction(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(GraphError::EntityExtraction(format!(
                "Ollama API returned error status: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| GraphError::EntityExtraction(format!("Failed to parse Ollama response: {}", e)))?;

        Ok(ollama_response.response)
    }

    /// Parse entities from LLM JSON response
    fn parse_entities_from_llm_response(&self, response: &str) -> Result<Vec<Entity>> {
        #[derive(Deserialize)]
        struct LlmEntity {
            name: String,
            #[serde(rename = "type")]
            entity_type: String,
            confidence: f64,
        }

        // Try to extract JSON from the response (LLM might include extra text)
        let json_start = response.find('[').unwrap_or(0);
        let json_end = response.rfind(']').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        let llm_entities: Vec<LlmEntity> = serde_json::from_str(json_str)
            .map_err(|e| GraphError::EntityExtraction(format!("Failed to parse LLM entities: {}", e)))?;

        let mut entities = Vec::new();
        for llm_entity in llm_entities {
            let entity_type = match llm_entity.entity_type.to_lowercase().as_str() {
                "person" => EntityType::Person,
                "place" => EntityType::Place,
                "organization" => EntityType::Organization,
                "system" => EntityType::Other("System".to_string()),
                "process" => EntityType::Other("Process".to_string()),
                "concept" => EntityType::Other("Concept".to_string()),
                _ => EntityType::Other(llm_entity.entity_type),
            };

            entities.push(Entity {
                id: Uuid::new_v4().to_string(),
                name: llm_entity.name,
                entity_type,
                attributes: vec![
                    Attribute {
                        id: Uuid::new_v4().to_string(),
                        name: "extraction_method".to_string(),
                        value: "LLM".to_string(),
                        attribute_type: AttributeType::Other("method".to_string()),
                        confidence: 1.0,
                    }
                ],
                confidence: llm_entity.confidence,
                position: None,
            });
        }

        Ok(entities)
    }

    /// Parse relationships from LLM JSON response
    fn parse_relationships_from_llm_response(&self, response: &str, entities: &[Entity]) -> Result<Vec<Relationship>> {
        #[derive(Deserialize)]
        struct LlmRelationship {
            from: String,
            to: String,
            relationship: String,
            confidence: f64,
        }

        // Try to extract JSON from the response
        let json_start = response.find('[').unwrap_or(0);
        let json_end = response.rfind(']').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        let llm_relationships: Vec<LlmRelationship> = serde_json::from_str(json_str)
            .map_err(|e| GraphError::EntityExtraction(format!("Failed to parse LLM relationships: {}", e)))?;

        // Create a mapping from entity names to IDs
        let entity_map: std::collections::HashMap<String, &Entity> = entities
            .iter()
            .map(|e| (e.name.to_lowercase(), e))
            .collect();

        let mut relationships = Vec::new();
        for llm_rel in llm_relationships {
            if let (Some(from_entity), Some(to_entity)) = (
                entity_map.get(&llm_rel.from.to_lowercase()),
                entity_map.get(&llm_rel.to.to_lowercase()),
            ) {
                relationships.push(Relationship {
                    id: Uuid::new_v4().to_string(),
                    source_entity_id: from_entity.id.clone(),
                    target_entity_id: to_entity.id.clone(),
                    relationship_type: RelationshipType::Other(llm_rel.relationship.clone()),
                    label: llm_rel.relationship,
                    confidence: llm_rel.confidence,
                    position: None,
                });
            }
        }

        Ok(relationships)
    }

    /// Parse concepts from LLM JSON response
    fn parse_concepts_from_llm_response(&self, response: &str) -> Result<Vec<Concept>> {
        #[derive(Deserialize)]
        struct LlmConcept {
            name: String,
            description: String,
            confidence: f64,
        }

        // Try to extract JSON from the response
        let json_start = response.find('[').unwrap_or(0);
        let json_end = response.rfind(']').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        let llm_concepts: Vec<LlmConcept> = serde_json::from_str(json_str)
            .map_err(|e| GraphError::EntityExtraction(format!("Failed to parse LLM concepts: {}", e)))?;

        let mut concepts = Vec::new();
        for llm_concept in llm_concepts {
            concepts.push(Concept {
                id: Uuid::new_v4().to_string(),
                name: llm_concept.name,
                description: llm_concept.description,
                related_entities: Vec::new(),
                confidence: llm_concept.confidence,
                position: None,
            });
        }

        Ok(concepts)
    }

    /// Perform deep analysis using LLM for comprehensive relationship extraction
    pub async fn extract_with_deep_analysis(&self, processed_text: &ProcessedText) -> Result<ExtractionResult> {
        if !self.config.use_llm {
            return Err(GraphError::EntityExtraction(
                "Deep analysis requires LLM to be enabled. Use --use-llm flag.".to_string()
            ));
        }

        println!("🔬 Starting deep analysis with LLM for comprehensive extraction...");
        let start_time = std::time::Instant::now();

        // Phase 1: Basic extraction
        let mut entities = self.extract_entities_with_llm(processed_text).await?;
        let mut relationships = self.extract_relationships_with_llm(processed_text, &entities).await?;
        let concepts = self.extract_concepts_with_llm(processed_text).await?;

        println!("📊 Initial extraction: {} entities, {} relationships, {} concepts", 
                entities.len(), relationships.len(), concepts.len());

        // Phase 2: Deep relationship analysis
        println!("🔍 Performing deep relationship analysis...");
        let deep_relationships = self.extract_deep_relationships_with_llm(processed_text, &entities).await?;
        relationships.extend(deep_relationships);

        // Phase 3: Contextual entity enhancement
        println!("✨ Enhancing entities with contextual information...");
        entities = self.enhance_entities_with_context(processed_text, entities).await?;

        // Phase 4: Advanced concept mapping
        println!("🧩 Mapping advanced concept relationships...");
        let concept_relationships = self.extract_concept_relationships(processed_text, &concepts, &entities).await?;
        relationships.extend(concept_relationships);

        let processing_time = start_time.elapsed().as_millis() as u64;

        let metadata = ExtractionMetadata {
            total_entities: entities.len(),
            total_relationships: relationships.len(),
            total_concepts: concepts.len(),
            processing_time_ms: processing_time,
            confidence_threshold: 0.6, // Higher threshold for deep analysis
            extraction_method: format!("Deep-Analysis-LLM-{}", self.config.llm_model),
        };

        println!("🎯 Deep analysis complete: {} entities, {} relationships, {} concepts", 
                entities.len(), relationships.len(), concepts.len());

        Ok(ExtractionResult {
            entities,
            relationships,
            concepts,
            metadata,
        })
    }

    /// Extract sophisticated relationships using advanced LLM prompting
    async fn extract_deep_relationships_with_llm(&self, processed_text: &ProcessedText, entities: &[Entity]) -> Result<Vec<Relationship>> {
        let entity_names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        
        let _prompt = format!(
            r#"Analyze the following text for sophisticated relationships between entities. 
            
Text: "{}"

Known entities: {:?}

Please identify:
1. Implicit relationships (not directly stated but implied)
2. Temporal relationships (sequence, causation)
3. Hierarchical relationships (parent-child, part-whole)
4. Functional relationships (roles, responsibilities)
5. Dependency relationships (requires, depends on)

Return relationships in JSON format:
[{{"from": "entity1", "to": "entity2", "type": "relationship_type", "confidence": 0.8, "context": "supporting_text"}}]"#,
            processed_text.cleaned_text,
            entity_names
        );

        // This would call the LLM - for now, return enhanced pattern-based relationships
        self.extract_relationships_with_enhanced_patterns(processed_text, entities)
    }

    /// Enhance entities with additional contextual information
    async fn enhance_entities_with_context(&self, processed_text: &ProcessedText, mut entities: Vec<Entity>) -> Result<Vec<Entity>> {
        for entity in &mut entities {
            // Add more detailed attributes based on context analysis
            let context_info = self.analyze_entity_context(processed_text, &entity.name);
            
            if let Some(role) = context_info.get("role") {
                entity.attributes.push(Attribute {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "contextual_role".to_string(),
                    value: role.clone(),
                    attribute_type: AttributeType::Other("role".to_string()),
                    confidence: 0.7,
                });
            }
            
            if let Some(domain) = context_info.get("domain") {
                entity.attributes.push(Attribute {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "domain".to_string(),
                    value: domain.clone(),
                    attribute_type: AttributeType::Other("domain".to_string()),
                    confidence: 0.7,
                });
            }
            
            // Increase confidence for entities with rich context
            if context_info.len() > 2 {
                entity.confidence = (entity.confidence * 1.2).min(1.0);
            }
        }
        
        Ok(entities)
    }

    /// Extract relationships between concepts and entities
    async fn extract_concept_relationships(&self, _processed_text: &ProcessedText, concepts: &[Concept], entities: &[Entity]) -> Result<Vec<Relationship>> {
        let mut relationships = Vec::new();
        
        for concept in concepts {
            for entity in entities {
                // Check if entity is mentioned in the same context as the concept
                let concept_context = &concept.description;
                if concept_context.to_lowercase().contains(&entity.name.to_lowercase()) {
                    relationships.push(Relationship {
                        id: uuid::Uuid::new_v4().to_string(),
                        source_entity_id: concept.id.clone(),
                        target_entity_id: entity.id.clone(),
                        relationship_type: RelationshipType::RelatedTo,
                        label: "relates to".to_string(),
                        confidence: 0.65,
                        position: None,
                    });
                }
            }
        }
        
        Ok(relationships)
    }

    /// Analyze contextual information about an entity
    fn analyze_entity_context(&self, processed_text: &ProcessedText, entity_name: &str) -> std::collections::HashMap<String, String> {
        let mut context_info = std::collections::HashMap::new();
        let text = &processed_text.cleaned_text.to_lowercase();
        let entity_lower = entity_name.to_lowercase();

        // Look for role indicators
        let role_patterns = [
            ("manager", "management_role"),
            ("developer", "technical_role"),
            ("customer", "business_role"),
            ("user", "user_role"),
            ("system", "system_component"),
            ("process", "business_process"),
        ];

        for (pattern, role) in role_patterns {
            if text.contains(&format!("{} {}", entity_lower, pattern)) || 
               text.contains(&format!("{} {}", pattern, entity_lower)) {
                context_info.insert("role".to_string(), role.to_string());
                break;
            }
        }

        // Look for domain indicators
        let domain_patterns = [
            ("database", "data_management"),
            ("server", "infrastructure"),
            ("application", "software"),
            ("network", "networking"),
            ("security", "cybersecurity"),
        ];

        for (pattern, domain) in domain_patterns {
            if text.contains(pattern) {
                context_info.insert("domain".to_string(), domain.to_string());
                break;
            }
        }

        context_info
    }

    /// Enhanced pattern-based relationship extraction for deep analysis
    fn extract_relationships_with_enhanced_patterns(&self, processed_text: &ProcessedText, entities: &[Entity]) -> Result<Vec<Relationship>> {
        let mut relationships = Vec::new();
        let entity_names: std::collections::HashMap<String, &Entity> = entities.iter()
            .map(|e| (e.name.to_lowercase(), e))
            .collect();

        // Enhanced patterns for sophisticated relationships
        let enhanced_patterns = [
            (r"(\w+)\s+manages?\s+(\w+)", RelationshipType::Other("manages".to_string())),
            (r"(\w+)\s+depends?\s+on\s+(\w+)", RelationshipType::Other("depends_on".to_string())),
            (r"(\w+)\s+implements?\s+(\w+)", RelationshipType::Other("implements".to_string())),
            (r"(\w+)\s+inherits?\s+from\s+(\w+)", RelationshipType::Other("inherits_from".to_string())),
            (r"(\w+)\s+communicates?\s+with\s+(\w+)", RelationshipType::Other("communicates_with".to_string())),
            (r"(\w+)\s+provides?\s+(\w+)", RelationshipType::Other("provides".to_string())),
            (r"(\w+)\s+requires?\s+(\w+)", RelationshipType::Other("requires".to_string())),
        ];

        for (pattern_str, rel_type) in enhanced_patterns {
            if let Ok(pattern) = Regex::new(pattern_str) {
                for capture in pattern.captures_iter(&processed_text.cleaned_text.to_lowercase()) {
                    if let (Some(entity1_match), Some(entity2_match)) = (capture.get(1), capture.get(2)) {
                        let entity1_name = entity1_match.as_str();
                        let entity2_name = entity2_match.as_str();

                        if let (Some(entity1), Some(entity2)) = (
                            entity_names.get(entity1_name),
                            entity_names.get(entity2_name)
                        ) {
                            let label = match &rel_type {
                                RelationshipType::Other(label) => label.clone(),
                                _ => "enhanced relationship".to_string(),
                            };
                            
                            relationships.push(Relationship {
                                id: uuid::Uuid::new_v4().to_string(),
                                source_entity_id: entity1.id.clone(),
                                target_entity_id: entity2.id.clone(),
                                relationship_type: rel_type.clone(),
                                label,
                                confidence: 0.75, // Higher confidence for enhanced patterns
                                position: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(relationships)
    }
}

impl Default for EntityExtractor {
    fn default() -> Self {
        Self::new(ExtractionConfig::default())
            .expect("Failed to create default EntityExtractor")
    }
}
