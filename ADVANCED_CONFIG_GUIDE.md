# MSG_NET Advanced Configuration Guide

## Overview
This guide explains the advanced configuration options available in MSG_NET for customizing entity relationship graph visualization and AI-enhanced text processing.

## Configuration Structure

The MSG_NET configuration follows a structured JSON format with five main sections:

### 1. Node Colors (`node_colors`)
Defines the color scheme for different types of nodes in the graph.

```json
"node_colors": {
  "entity": "#FF6B6B",      // Red for entities (people, places, things)
  "relationship": "#4ECDC4", // Teal for relationship nodes
  "concept": "#45B7D1",     // Blue for abstract concepts
  "attribute": "#FFA07A"    // Orange for attributes and properties
}
```

**Supported formats:**
- Hex colors: `#FF6B6B`
- RGB colors: `rgb(255, 107, 107)`
- Named colors: `red`, `blue`, `green`

### 2. Node Shapes (`node_shapes`)
Specifies the visual shape for each node type.

```json
"node_shapes": {
  "entity": "ellipse",      // Oval shape for entities
  "relationship": "box",    // Rectangle for relationships
  "concept": "circle",      // Circle for concepts
  "attribute": "diamond"    // Diamond for attributes
}
```

**Available shapes:**
- `ellipse` - Oval/elliptical
- `circle` - Perfect circle
- `box` - Rectangle/square
- `diamond` - Diamond shape
- `triangle` - Triangle
- `star` - Star shape
- `dot` - Small dot

### 3. Layout Configuration (`layout`)
Controls the overall arrangement and positioning of nodes.

```json
"layout": {
  "algorithm": "hierarchical",  // Layout algorithm to use
  "spacing": 300.0,            // Distance between nodes
  "hierarchical": true         // Enable hierarchical layout
}
```

**Layout algorithms:**
- `hierarchical` - Tree-like structure with clear hierarchy
- `force` - Force-directed layout using physics simulation
- `circular` - Nodes arranged in circular pattern
- `grid` - Regular grid arrangement

**Spacing values:**
- `100.0-500.0` - Recommended range for optimal visibility
- Higher values = more spread out
- Lower values = more compact

### 4. Physics Configuration (`physics`)
Controls the physics simulation for dynamic layouts.

```json
"physics": {
  "enabled": true,           // Enable/disable physics simulation
  "stabilization": true,     // Auto-stabilize the layout
  "repulsion": 250.0,       // Force pushing nodes apart
  "spring_length": 200.0,   // Preferred edge length
  "spring_constant": 0.05   // Edge stiffness
}
```

**Physics parameters:**
- `repulsion`: `50.0-500.0` - Higher values spread nodes further apart
- `spring_length`: `50.0-400.0` - Target length for edges
- `spring_constant`: `0.01-0.1` - Edge flexibility (lower = more flexible)

### 5. Extraction Configuration (`extraction`)
Advanced settings for AI-powered entity and relationship extraction.

```json
"extraction": {
  "use_llm": true,
  "llm_model": "llama3.2",
  "llm_endpoint": "http://localhost:11434/api/generate",
  "entity_patterns": [...],
  "relationship_patterns": [...],
  "concept_patterns": [...]
}
```

#### LLM Settings
- `use_llm`: Enable AI-powered extraction for better accuracy
- `llm_model`: Specify the model (llama3.2, llama2, codellama, etc.)
- `llm_endpoint`: Ollama API endpoint (default: localhost:11434)

#### Pattern Arrays
Advanced regex patterns for enhanced text processing:

**Entity Patterns:**
```json
"entity_patterns": [
  "\\b[A-Z][a-z]+(?:\\s+[A-Z][a-z]+)*\\b",  // Proper nouns
  "\\b(?:Dr\\.|Mr\\.|Ms\\.|Mrs\\.|Prof\\.)\\s+[A-Z][a-z]+\\b",  // Titles
  "\\b(?:organization|company|institution)\\b"  // Organization keywords
]
```

**Relationship Patterns:**
```json
"relationship_patterns": [
  "\\b(?:has|have|is|are|owns|belongs|manages)\\b",  // Possession/ownership
  "\\b(?:connected to|related to|depends on)\\b",    // Connections
  "\\b(?:creates|develops|builds|implements)\\b"     // Actions
]
```

**Concept Patterns:**
```json
"concept_patterns": [
  "\\b(?:concept|idea|principle|theory)\\b",        // Abstract concepts
  "\\b(?:goal|objective|target|purpose)\\b",        // Objectives
  "\\b(?:technology|framework|architecture)\\b"     // Technical concepts
]
```

## Usage Examples

### Basic Configuration
Minimal configuration for simple graphs:
```json
{
  "node_colors": {
    "entity": "#FF6B6B",
    "relationship": "#4ECDC4", 
    "concept": "#45B7D1",
    "attribute": "#FFA07A"
  },
  "node_shapes": {
    "entity": "ellipse",
    "relationship": "box",
    "concept": "circle", 
    "attribute": "diamond"
  },
  "layout": {
    "algorithm": "hierarchical",
    "spacing": 200.0,
    "hierarchical": true
  },
  "physics": {
    "enabled": true,
    "stabilization": true,
    "repulsion": 200.0,
    "spring_length": 150.0,
    "spring_constant": 0.04
  },
  "extraction": {
    "use_llm": false,
    "llm_model": "llama3.2",
    "llm_endpoint": "http://localhost:11434/api/generate",
    "entity_patterns": [],
    "relationship_patterns": [],
    "concept_patterns": []
  }
}
```

### Advanced AI-Enhanced Configuration
Full configuration with LLM and custom patterns:
```bash
# Use the advanced_config_fixed.json file created above
cargo run -- generate -i input.txt -o output.html -c advanced_config_fixed.json --use-llm
```

## Best Practices

### Color Selection
- Use high contrast colors for better visibility
- Consider colorblind-friendly palettes
- Maintain consistency across related projects

### Physics Tuning
- Start with default values and adjust incrementally
- Higher repulsion for dense graphs
- Lower spring constants for more organic layouts

### Pattern Development
- Test patterns with sample text first
- Use specific patterns before general ones
- Balance precision vs. recall in extraction

### Performance Optimization
- Disable physics for large graphs (>100 nodes)
- Use simpler shapes for better rendering performance
- Consider hierarchical layout for complex relationships

## Troubleshooting

### Common Issues
1. **JSON Parsing Errors**: Validate JSON syntax and structure
2. **Missing LLM**: Ensure Ollama is running on specified endpoint
3. **Poor Extraction**: Refine patterns or enable LLM processing
4. **Layout Issues**: Adjust spacing and physics parameters

### Validation
Always test configuration changes:
```bash
# Validate configuration
cargo run -- generate -i test.txt -o test.html -c your_config.json

# Check for errors in the output
tail -f msg_net.log
```

## Integration Examples

### Command Line Usage
```bash
# Basic usage with advanced config
cargo run -- generate -i document.txt -o graph.html -c advanced_config_fixed.json

# With deep analysis
cargo run -- generate -i document.txt -o graph.html -c advanced_config_fixed.json --use-llm --deep-analysis

# Custom LLM model
cargo run -- generate -i document.txt -o graph.html -c advanced_config_fixed.json --use-llm --llm-model llama2
```

### Batch Processing
```bash
# Process multiple files with same config
for file in *.txt; do
  cargo run -- generate -i "$file" -o "${file%.txt}_graph.html" -c advanced_config_fixed.json --use-llm
done
```

This advanced configuration system enables fine-tuned control over every aspect of MSG_NET's visualization and AI processing capabilities.
