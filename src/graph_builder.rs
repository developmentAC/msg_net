use crate::config::GraphConfig;
use crate::entity_extractor::{Entity, Relationship, Concept, ExtractionResult};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub color: String,
    pub shape: String,
    pub size: f64,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub physics: bool,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub label: String,
    pub color: String,
    pub width: f64,
    pub arrows: String,
    pub edge_type: EdgeType,
    pub metadata: EdgeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Entity,
    Concept,
    Attribute,
    Relationship,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    EntityRelationship,
    EntityAttribute,
    ConceptEntity,
    ConceptConcept,
    Hierarchy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub confidence: f64,
    pub original_text: String,
    pub entity_type: Option<String>,
    pub attributes: HashMap<String, String>,
    pub position_in_text: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    pub confidence: f64,
    pub relationship_type: String,
    pub bidirectional: bool,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub config: GraphConfig,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub node_types: HashMap<String, usize>,
    pub edge_types: HashMap<String, usize>,
    pub creation_timestamp: String,
    pub source_text_length: usize,
}

pub struct GraphBuilder {
    config: GraphConfig,
}

impl GraphBuilder {
    pub fn new(config: GraphConfig) -> Self {
        Self { config }
    }

    pub fn build_graph(&self, extraction_result: &ExtractionResult, source_text: &str) -> Result<InteractiveGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_types = HashMap::new();
        let mut edge_types = HashMap::new();

        // Build entity nodes
        for entity in &extraction_result.entities {
            let node = self.create_entity_node(entity)?;
            *node_types.entry("entity".to_string()).or_insert(0) += 1;
            nodes.push(node);

            // Create attribute nodes and edges
            for attribute in &entity.attributes {
                if attribute.name != "name" { // Skip name attribute as it's already the entity label
                    let attr_node = self.create_attribute_node(entity, attribute)?;
                    let attr_edge = self.create_attribute_edge(entity, attribute)?;
                    
                    *node_types.entry("attribute".to_string()).or_insert(0) += 1;
                    *edge_types.entry("entity_attribute".to_string()).or_insert(0) += 1;
                    
                    nodes.push(attr_node);
                    edges.push(attr_edge);
                }
            }
        }

        // Build concept nodes
        for concept in &extraction_result.concepts {
            let node = self.create_concept_node(concept)?;
            *node_types.entry("concept".to_string()).or_insert(0) += 1;
            nodes.push(node);
        }

        // Build relationship edges
        for relationship in &extraction_result.relationships {
            let edge = self.create_relationship_edge(relationship)?;
            *edge_types.entry("relationship".to_string()).or_insert(0) += 1;
            edges.push(edge);
        }

        // Create concept-entity connections
        self.create_concept_entity_connections(&extraction_result.concepts, &extraction_result.entities, &mut edges, &mut edge_types)?;

        let metadata = GraphMetadata {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            node_types,
            edge_types,
            creation_timestamp: chrono::Utc::now().to_rfc3339(),
            source_text_length: source_text.len(),
        };

        Ok(InteractiveGraph {
            nodes,
            edges,
            config: self.config.clone(),
            metadata,
        })
    }

    fn create_entity_node(&self, entity: &Entity) -> Result<GraphNode> {
        let metadata = NodeMetadata {
            confidence: entity.confidence,
            original_text: entity.name.clone(),
            entity_type: Some(format!("{:?}", entity.entity_type)),
            attributes: entity.attributes.iter()
                .map(|attr| (attr.name.clone(), attr.value.clone()))
                .collect(),
            position_in_text: entity.position.as_ref()
                .map(|pos| (pos.start, pos.end)),
        };

        Ok(GraphNode {
            id: entity.id.clone(),
            label: entity.name.clone(),
            node_type: NodeType::Entity,
            color: self.config.node_colors.entity.clone(),
            shape: self.config.node_shapes.entity.clone(),
            size: self.calculate_node_size(entity.confidence, &entity.attributes),
            x: None,
            y: None,
            physics: true,
            metadata,
        })
    }

    fn create_concept_node(&self, concept: &Concept) -> Result<GraphNode> {
        let metadata = NodeMetadata {
            confidence: concept.confidence,
            original_text: concept.name.clone(),
            entity_type: Some("concept".to_string()),
            attributes: [
                ("description".to_string(), concept.description.clone()),
                ("related_entities_count".to_string(), concept.related_entities.len().to_string()),
            ].iter().cloned().collect(),
            position_in_text: concept.position.as_ref()
                .map(|pos| (pos.start, pos.end)),
        };

        Ok(GraphNode {
            id: concept.id.clone(),
            label: concept.name.clone(),
            node_type: NodeType::Concept,
            color: self.config.node_colors.concept.clone(),
            shape: self.config.node_shapes.concept.clone(),
            size: self.calculate_concept_node_size(concept.confidence, concept.related_entities.len()),
            x: None,
            y: None,
            physics: true,
            metadata,
        })
    }

    fn create_attribute_node(&self, entity: &Entity, attribute: &crate::entity_extractor::Attribute) -> Result<GraphNode> {
        let metadata = NodeMetadata {
            confidence: attribute.confidence,
            original_text: attribute.value.clone(),
            entity_type: Some(format!("{:?}", attribute.attribute_type)),
            attributes: [
                ("attribute_name".to_string(), attribute.name.clone()),
                ("parent_entity".to_string(), entity.name.clone()),
            ].iter().cloned().collect(),
            position_in_text: None,
        };

        Ok(GraphNode {
            id: attribute.id.clone(),
            label: format!("{}: {}", attribute.name, attribute.value),
            node_type: NodeType::Attribute,
            color: self.config.node_colors.attribute.clone(),
            shape: self.config.node_shapes.attribute.clone(),
            size: 20.0, // Fixed size for attributes
            x: None,
            y: None,
            physics: true,
            metadata,
        })
    }

    fn create_relationship_edge(&self, relationship: &Relationship) -> Result<GraphEdge> {
        let metadata = EdgeMetadata {
            confidence: relationship.confidence,
            relationship_type: format!("{:?}", relationship.relationship_type),
            bidirectional: false, // Can be enhanced based on relationship type
            weight: relationship.confidence,
        };

        Ok(GraphEdge {
            id: relationship.id.clone(),
            from: relationship.source_entity_id.clone(),
            to: relationship.target_entity_id.clone(),
            label: relationship.label.clone(),
            color: self.config.node_colors.relationship.clone(),
            width: self.calculate_edge_width(relationship.confidence),
            arrows: "to".to_string(),
            edge_type: EdgeType::EntityRelationship,
            metadata,
        })
    }

    fn create_attribute_edge(&self, entity: &Entity, attribute: &crate::entity_extractor::Attribute) -> Result<GraphEdge> {
        let metadata = EdgeMetadata {
            confidence: attribute.confidence,
            relationship_type: "has_attribute".to_string(),
            bidirectional: false,
            weight: attribute.confidence,
        };

        Ok(GraphEdge {
            id: format!("{}-{}", entity.id, attribute.id),
            from: entity.id.clone(),
            to: attribute.id.clone(),
            label: "has".to_string(),
            color: "#888888".to_string(),
            width: 1.0,
            arrows: "to".to_string(),
            edge_type: EdgeType::EntityAttribute,
            metadata,
        })
    }

    fn create_concept_entity_connections(
        &self,
        concepts: &[Concept],
        entities: &[Entity],
        edges: &mut Vec<GraphEdge>,
        edge_types: &mut HashMap<String, usize>,
    ) -> Result<()> {
        for concept in concepts {
            // Simple heuristic: connect concepts to entities that appear in the same context
            for entity in entities {
                if self.should_connect_concept_to_entity(concept, entity) {
                    let edge = self.create_concept_entity_edge(concept, entity)?;
                    *edge_types.entry("concept_entity".to_string()).or_insert(0) += 1;
                    edges.push(edge);
                }
            }
        }
        Ok(())
    }

    fn should_connect_concept_to_entity(&self, concept: &Concept, entity: &Entity) -> bool {
        // Connect if they appear in similar text positions or have semantic similarity
        if let (Some(concept_pos), Some(entity_pos)) = (&concept.position, &entity.position) {
            // Connect if they're in the same sentence or adjacent sentences
            concept_pos.sentence_index.abs_diff(entity_pos.sentence_index) <= 1
        } else {
            // Fallback: simple text matching
            concept.description.to_lowercase().contains(&entity.name.to_lowercase()) ||
            entity.name.to_lowercase().contains(&concept.name.to_lowercase())
        }
    }

    fn create_concept_entity_edge(&self, concept: &Concept, entity: &Entity) -> Result<GraphEdge> {
        let metadata = EdgeMetadata {
            confidence: (concept.confidence + entity.confidence) / 2.0,
            relationship_type: "related_to".to_string(),
            bidirectional: true,
            weight: 0.5,
        };

        Ok(GraphEdge {
            id: format!("{}-{}", concept.id, entity.id),
            from: concept.id.clone(),
            to: entity.id.clone(),
            label: "relates to".to_string(),
            color: "#CCCCCC".to_string(),
            width: 1.0,
            arrows: "to".to_string(),
            edge_type: EdgeType::ConceptEntity,
            metadata,
        })
    }

    fn calculate_node_size(&self, confidence: f64, attributes: &[crate::entity_extractor::Attribute]) -> f64 {
        let base_size = 30.0;
        let confidence_factor = 1.0 + confidence * 0.5;
        let attribute_factor = 1.0 + (attributes.len() as f64 * 0.1);
        
        base_size * confidence_factor * attribute_factor
    }

    fn calculate_concept_node_size(&self, confidence: f64, related_entities_count: usize) -> f64 {
        let base_size = 25.0;
        let confidence_factor = 1.0 + confidence * 0.3;
        let relation_factor = 1.0 + (related_entities_count as f64 * 0.15);
        
        base_size * confidence_factor * relation_factor
    }

    fn calculate_edge_width(&self, confidence: f64) -> f64 {
        1.0 + confidence * 2.0
    }

    pub fn apply_layout(&self, graph: &mut InteractiveGraph) -> Result<()> {
        match self.config.layout.algorithm.as_str() {
            "hierarchical" => self.apply_hierarchical_layout(graph),
            "force" => self.apply_force_layout(graph),
            "circular" => self.apply_circular_layout(graph),
            _ => self.apply_force_layout(graph), // Default to force layout
        }
    }

    fn apply_hierarchical_layout(&self, graph: &mut InteractiveGraph) -> Result<()> {
        // Simple hierarchical layout: entities at top, concepts in middle, attributes at bottom
        let mut entity_nodes = Vec::new();
        let mut concept_nodes = Vec::new();
        let mut attribute_nodes = Vec::new();

        for node in &mut graph.nodes {
            match node.node_type {
                NodeType::Entity => entity_nodes.push(node),
                NodeType::Concept => concept_nodes.push(node),
                NodeType::Attribute => attribute_nodes.push(node),
                NodeType::Relationship => {}, // Relationships are represented as edges
            }
        }

        // Position entities
        let entity_y = -200.0;
        let entity_count = entity_nodes.len();
        for (i, node) in entity_nodes.iter_mut().enumerate() {
            node.x = Some((i as f64 - entity_count as f64 / 2.0) * self.config.layout.spacing);
            node.y = Some(entity_y);
        }

        // Position concepts
        let concept_y = 0.0;
        let concept_count = concept_nodes.len();
        for (i, node) in concept_nodes.iter_mut().enumerate() {
            node.x = Some((i as f64 - concept_count as f64 / 2.0) * self.config.layout.spacing);
            node.y = Some(concept_y);
        }

        // Position attributes
        let attribute_y = 200.0;
        let attribute_count = attribute_nodes.len();
        for (i, node) in attribute_nodes.iter_mut().enumerate() {
            node.x = Some((i as f64 - attribute_count as f64 / 2.0) * self.config.layout.spacing);
            node.y = Some(attribute_y);
        }

        Ok(())
    }

    fn apply_force_layout(&self, _graph: &mut InteractiveGraph) -> Result<()> {
        // For force layout, we let vis.js handle the positioning
        // Just ensure physics is enabled for all nodes
        Ok(())
    }

    fn apply_circular_layout(&self, graph: &mut InteractiveGraph) -> Result<()> {
        use std::f64::consts::PI;
        
        let node_count = graph.nodes.len();
        if node_count == 0 {
            return Ok(());
        }

        let radius = 300.0;
        let angle_step = 2.0 * PI / node_count as f64;

        for (i, node) in graph.nodes.iter_mut().enumerate() {
            let angle = i as f64 * angle_step;
            node.x = Some(radius * angle.cos());
            node.y = Some(radius * angle.sin());
        }

        Ok(())
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new(GraphConfig::default())
    }
}
