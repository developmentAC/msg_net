use crate::graph_builder::InteractiveGraph;
use crate::web_interface::WebInterface;
use crate::error::{GraphError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub include_styling: bool,
    pub compact_output: bool,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Html,
    Json,
    Csv,
    GraphML,
    Dot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub file_path: Option<String>,
    pub content: Option<String>,
    pub error_message: Option<String>,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub export_timestamp: String,
    pub original_graph_nodes: usize,
    pub original_graph_edges: usize,
    pub exported_format: String,
    pub file_size_bytes: Option<usize>,
}

pub struct GraphExporter {
    web_interface: WebInterface,
}

impl GraphExporter {
    pub fn new() -> Self {
        Self {
            web_interface: WebInterface::new("export-container".to_string()),
        }
    }

    /// Create serialized filename in the 0_networks directory
    fn create_output_path(&self, requested_path: &str) -> Result<String> {
        let path = Path::new(requested_path);
        let filename = path.file_name()
            .ok_or_else(|| GraphError::Export("Invalid filename".to_string()))?;
        let stem = path.file_stem()
            .ok_or_else(|| GraphError::Export("Invalid file stem".to_string()))?;
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| GraphError::Export("Invalid file extension".to_string()))?;
        
        // Create 0_networks directory if it doesn't exist
        let networks_dir = Path::new("0_networks");
        if !networks_dir.exists() {
            fs::create_dir_all(networks_dir)
                .map_err(|e| GraphError::Export(format!("Failed to create directory: {}", e)))?;
        }
        
        // Generate serialized filename
        let mut counter = 0;
        let mut output_path = networks_dir.join(filename);
        
        while output_path.exists() {
            counter += 1;
            let serialized_name = format!("{}_{:02}.{}", 
                stem.to_string_lossy(), 
                counter, 
                extension
            );
            output_path = networks_dir.join(serialized_name);
        }
        
        Ok(output_path.to_string_lossy().to_string())
    }

    pub fn export_graph(&self, graph: &InteractiveGraph, options: &ExportOptions) -> Result<ExportResult> {
        match options.format {
            ExportFormat::Html => self.export_to_html(graph, options),
            ExportFormat::Json => self.export_to_json(graph, options),
            ExportFormat::Csv => self.export_to_csv(graph, options),
            ExportFormat::GraphML => self.export_to_graphml(graph, options),
            ExportFormat::Dot => self.export_to_dot(graph, options),
        }
    }

    fn export_to_html(&self, graph: &InteractiveGraph, options: &ExportOptions) -> Result<ExportResult> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        // Create output path with serialization
        let output_path = if let Some(path) = &options.file_path {
            self.create_output_path(path)?
        } else {
            self.create_output_path("graph.html")?
        };
        
        // Create the HTML content with embedded vis.js
        let title = "Entity Relationship Graph";
        let html_template = self.web_interface.create_html_template(title);
        
        // Embed the graph data directly in the HTML
        let nodes_json = serde_json::to_string(&graph.nodes)?;
        let edges_json = serde_json::to_string(&graph.edges)?;
        let config_json = serde_json::to_string(&graph.config)?;
        
        let embedded_script = format!(r#"
        <script>
            // Graph data embedded directly in HTML
            window.graphData = {{
                nodes: {},
                edges: {},
                config: {}
            }};
            
            // Initialize the graph when page loads
            window.addEventListener('load', function() {{
                initializeGraph();
            }});
            
            function initializeGraph() {{
                // Sync physics enabled state with config
                physicsEnabled = window.graphData.config.physics.enabled;
                
                const container = document.getElementById('{}');
                const nodes = new vis.DataSet(window.graphData.nodes.map(node => ({{
                    id: node.id,
                    label: node.label,
                    originalLabel: node.label, // Store original label for toggle functionality
                    color: node.color,
                    shape: node.shape,
                    size: node.size,
                    x: node.x,
                    y: node.y,
                    physics: node.physics,
                    title: `Type: ${{node.node_type}}<br/>Confidence: ${{node.metadata.confidence.toFixed(2)}}`,
                    group: node.node_type.toLowerCase(),
                    node_type: node.node_type,
                    confidence: node.metadata.confidence
                }})));
                
                const edges = new vis.DataSet(window.graphData.edges.map(edge => ({{
                    id: edge.id,
                    from: edge.from,
                    to: edge.to,
                    label: edge.label,
                    originalLabel: edge.label, // Store original label for toggle functionality
                    color: edge.color,
                    width: edge.width,
                    arrows: edge.arrows,
                    title: `Type: ${{edge.metadata.relationship_type}}<br/>Confidence: ${{edge.metadata.confidence.toFixed(2)}}`,
                    smooth: {{ type: "continuous" }},
                    relationship_type: edge.metadata.relationship_type
                }})));
                
                // Store original data globally for filtering and label toggling
                originalNodes = nodes.get();
                originalEdges = edges.get();
                
                const data = {{ nodes: nodes, edges: edges }};
                
                const options = {{
                    nodes: {{
                        shape: 'dot',
                        size: 25,
                        font: {{
                            size: 14,
                            color: '#343434',
                            face: 'arial'
                        }},
                        borderWidth: 2,
                        shadow: true
                    }},
                    edges: {{
                        width: 2,
                        arrows: {{
                            to: {{
                                enabled: true,
                                scaleFactor: 1
                            }}
                        }},
                        smooth: true,
                        shadow: true
                    }},
                    physics: {{
                        enabled: window.graphData.config.physics.enabled,
                        stabilization: {{
                            enabled: window.graphData.config.physics.stabilization,
                            iterations: 1000
                        }},
                        repulsion: {{
                            nodeDistance: window.graphData.config.physics.repulsion,
                            centralGravity: 0.1,
                            springLength: window.graphData.config.physics.spring_length,
                            springConstant: window.graphData.config.physics.spring_constant
                        }}
                    }},
                    interaction: {{
                        dragNodes: true,
                        dragView: true,
                        zoomView: true,
                        selectConnectedEdges: true,
                        hover: true
                    }}
                }};
                
                // Assign to the global variable (not window.currentNetwork)
                currentNetwork = new vis.Network(container, data, options);
                
                // Set up event listeners
                currentNetwork.on('selectNode', function(params) {{
                    onNodeSelected(params.nodes[0]);
                }});
                
                currentNetwork.on('selectEdge', function(params) {{
                    onEdgeSelected(params.edges[0]);
                }});
                
                // Initialize toggle button states
                updateToggleButton('physicsToggle', physicsEnabled, 'Physics: ON', 'Physics: OFF');
                updateToggleButton('nodeLabelsToggle', showNodeLabels, 'Node Labels: ON', 'Node Labels: OFF');
                updateToggleButton('edgeLabelsToggle', showEdgeLabels, 'Edge Labels: ON', 'Edge Labels: OFF');
                
                console.log('Graph initialized successfully');
            }}
        </script>
        "#, nodes_json, edges_json, config_json, self.web_interface.get_container_id());
        
        // Insert the script before the closing body tag
        let final_html = html_template.replace("</body>", &format!("{}\n</body>", embedded_script));
        
        let metadata = ExportMetadata {
            export_timestamp: timestamp,
            original_graph_nodes: graph.nodes.len(),
            original_graph_edges: graph.edges.len(),
            exported_format: "HTML".to_string(),
            file_size_bytes: Some(final_html.len()),
        };
        
        // Write to file
        fs::write(&output_path, &final_html)
            .map_err(|e| GraphError::Export(format!("Failed to write HTML file: {}", e)))?;
        
        Ok(ExportResult {
            success: true,
            file_path: Some(output_path),
            content: if options.compact_output { None } else { Some(final_html) },
            error_message: None,
            metadata,
        })
    }

    fn export_to_json(&self, graph: &InteractiveGraph, options: &ExportOptions) -> Result<ExportResult> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        // Create output path with serialization
        let output_path = if let Some(path) = &options.file_path {
            self.create_output_path(path)?
        } else {
            self.create_output_path("graph.json")?
        };
        
        let json_data = if options.include_metadata {
            serde_json::to_string_pretty(graph)?
        } else {
            // Export only nodes and edges
            let simplified = serde_json::json!({
                "nodes": graph.nodes,
                "edges": graph.edges
            });
            if options.compact_output {
                serde_json::to_string(&simplified)?
            } else {
                serde_json::to_string_pretty(&simplified)?
            }
        };
        
        let metadata = ExportMetadata {
            export_timestamp: timestamp,
            original_graph_nodes: graph.nodes.len(),
            original_graph_edges: graph.edges.len(),
            exported_format: "JSON".to_string(),
            file_size_bytes: Some(json_data.len()),
        };
        
        fs::write(&output_path, &json_data)
            .map_err(|e| GraphError::Export(format!("Failed to write JSON file: {}", e)))?;
        
        Ok(ExportResult {
            success: true,
            file_path: Some(output_path),
            content: if options.compact_output { None } else { Some(json_data) },
            error_message: None,
            metadata,
        })
    }

    fn export_to_csv(&self, graph: &InteractiveGraph, options: &ExportOptions) -> Result<ExportResult> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        // Create output path with serialization
        let output_path = if let Some(path) = &options.file_path {
            self.create_output_path(path)?
        } else {
            self.create_output_path("graph.csv")?
        };
        
        // Create separate CSV sections for nodes and edges
        let mut csv_content = String::new();
        
        // Nodes section
        csv_content.push_str("# NODES\n");
        csv_content.push_str("id,label,type,color,shape,size,confidence\n");
        
        for node in &graph.nodes {
            csv_content.push_str(&format!(
                "{},{},{:?},{},{},{},{}\n",
                node.id,
                node.label.replace(',', ";"), // Escape commas
                node.node_type,
                node.color,
                node.shape,
                node.size,
                node.metadata.confidence
            ));
        }
        
        // Edges section
        csv_content.push_str("\n# EDGES\n");
        csv_content.push_str("id,from,to,label,type,color,width,confidence\n");
        
        for edge in &graph.edges {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                edge.id,
                edge.from,
                edge.to,
                edge.label.replace(',', ";"), // Escape commas
                format!("{:?}", edge.edge_type),
                edge.color,
                edge.width,
                edge.metadata.confidence
            ));
        }
        
        let metadata = ExportMetadata {
            export_timestamp: timestamp,
            original_graph_nodes: graph.nodes.len(),
            original_graph_edges: graph.edges.len(),
            exported_format: "CSV".to_string(),
            file_size_bytes: Some(csv_content.len()),
        };
        
        fs::write(&output_path, &csv_content)
            .map_err(|e| GraphError::Export(format!("Failed to write CSV file: {}", e)))?;
        
        Ok(ExportResult {
            success: true,
            file_path: Some(output_path),
            content: if options.compact_output { None } else { Some(csv_content) },
            error_message: None,
            metadata,
        })
    }

    fn export_to_graphml(&self, graph: &InteractiveGraph, options: &ExportOptions) -> Result<ExportResult> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        // Create output path with serialization
        let output_path = if let Some(path) = &options.file_path {
            self.create_output_path(path)?
        } else {
            self.create_output_path("graph.graphml")?
        };
        
        let mut graphml_content = String::new();
        
        // GraphML header
        graphml_content.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://graphml.graphdrawing.org/xmlns 
         http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd">

"#);
        
        // Define attributes
        graphml_content.push_str(r#"  <key id="d0" for="node" attr.name="label" attr.type="string"/>
  <key id="d1" for="node" attr.name="type" attr.type="string"/>
  <key id="d2" for="node" attr.name="confidence" attr.type="double"/>
  <key id="d3" for="edge" attr.name="label" attr.type="string"/>
  <key id="d4" for="edge" attr.name="type" attr.type="string"/>
  <key id="d5" for="edge" attr.name="confidence" attr.type="double"/>

"#);
        
        // Graph element
        graphml_content.push_str("  <graph id=\"G\" edgedefault=\"directed\">\n");
        
        // Nodes
        for node in &graph.nodes {
            graphml_content.push_str(&format!(
                "    <node id=\"{}\">\n",
                Self::escape_xml(&node.id)
            ));
            graphml_content.push_str(&format!(
                "      <data key=\"d0\">{}</data>\n",
                Self::escape_xml(&node.label)
            ));
            graphml_content.push_str(&format!(
                "      <data key=\"d1\">{:?}</data>\n",
                node.node_type
            ));
            graphml_content.push_str(&format!(
                "      <data key=\"d2\">{}</data>\n",
                node.metadata.confidence
            ));
            graphml_content.push_str("    </node>\n");
        }
        
        // Edges
        for edge in &graph.edges {
            graphml_content.push_str(&format!(
                "    <edge id=\"{}\" source=\"{}\" target=\"{}\">\n",
                Self::escape_xml(&edge.id),
                Self::escape_xml(&edge.from),
                Self::escape_xml(&edge.to)
            ));
            graphml_content.push_str(&format!(
                "      <data key=\"d3\">{}</data>\n",
                Self::escape_xml(&edge.label)
            ));
            graphml_content.push_str(&format!(
                "      <data key=\"d4\">{:?}</data>\n",
                edge.edge_type
            ));
            graphml_content.push_str(&format!(
                "      <data key=\"d5\">{}</data>\n",
                edge.metadata.confidence
            ));
            graphml_content.push_str("    </edge>\n");
        }
        
        // Close graph and graphml
        graphml_content.push_str("  </graph>\n");
        graphml_content.push_str("</graphml>\n");
        
        let metadata = ExportMetadata {
            export_timestamp: timestamp,
            original_graph_nodes: graph.nodes.len(),
            original_graph_edges: graph.edges.len(),
            exported_format: "GraphML".to_string(),
            file_size_bytes: Some(graphml_content.len()),
        };
        
        fs::write(&output_path, &graphml_content)
            .map_err(|e| GraphError::Export(format!("Failed to write GraphML file: {}", e)))?;
        
        Ok(ExportResult {
            success: true,
            file_path: Some(output_path),
            content: if options.compact_output { None } else { Some(graphml_content) },
            error_message: None,
            metadata,
        })
    }

    fn export_to_dot(&self, graph: &InteractiveGraph, options: &ExportOptions) -> Result<ExportResult> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        // Create output path with serialization
        let output_path = if let Some(path) = &options.file_path {
            self.create_output_path(path)?
        } else {
            self.create_output_path("graph.dot")?
        };
        
        let mut dot_content = String::new();
        
        // DOT header
        dot_content.push_str("digraph EntityRelationshipGraph {\n");
        dot_content.push_str("  rankdir=TB;\n");
        dot_content.push_str("  node [shape=ellipse, style=filled];\n");
        dot_content.push_str("  edge [fontsize=10];\n\n");
        
        // Nodes
        for node in &graph.nodes {
            let shape = match node.node_type {
                crate::graph_builder::NodeType::Entity => "ellipse",
                crate::graph_builder::NodeType::Concept => "circle",
                crate::graph_builder::NodeType::Attribute => "box",
                crate::graph_builder::NodeType::Relationship => "diamond",
            };
            
            dot_content.push_str(&format!(
                "  \"{}\" [label=\"{}\", shape={}, fillcolor=\"{}\", tooltip=\"Confidence: {:.2}\"];\n",
                Self::escape_dot(&node.id),
                Self::escape_dot(&node.label),
                shape,
                node.color,
                node.metadata.confidence
            ));
        }
        
        dot_content.push_str("\n");
        
        // Edges
        for edge in &graph.edges {
            dot_content.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\", color=\"{}\", penwidth={}, tooltip=\"Confidence: {:.2}\"];\n",
                Self::escape_dot(&edge.from),
                Self::escape_dot(&edge.to),
                Self::escape_dot(&edge.label),
                edge.color,
                edge.width,
                edge.metadata.confidence
            ));
        }
        
        dot_content.push_str("}\n");
        
        let metadata = ExportMetadata {
            export_timestamp: timestamp,
            original_graph_nodes: graph.nodes.len(),
            original_graph_edges: graph.edges.len(),
            exported_format: "DOT".to_string(),
            file_size_bytes: Some(dot_content.len()),
        };
        
        fs::write(&output_path, &dot_content)
            .map_err(|e| GraphError::Export(format!("Failed to write DOT file: {}", e)))?;
        
        Ok(ExportResult {
            success: true,
            file_path: Some(output_path),
            content: if options.compact_output { None } else { Some(dot_content) },
            error_message: None,
            metadata,
        })
    }

    fn escape_xml(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn escape_dot(text: &str) -> String {
        text.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    pub fn get_supported_formats() -> Vec<ExportFormat> {
        vec![
            ExportFormat::Html,
            ExportFormat::Json,
            ExportFormat::Csv,
            ExportFormat::GraphML,
            ExportFormat::Dot,
        ]
    }

    pub fn validate_export_path(file_path: &str, format: &ExportFormat) -> Result<()> {
        let path = Path::new(file_path);
        
        // Check if the directory exists (skip check for current directory)
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(GraphError::Export(format!(
                    "Directory does not exist: {}",
                    parent.display()
                )));
            }
        }
        
        // Check file extension matches format
        let expected_extension = match format {
            ExportFormat::Html => "html",
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::GraphML => "graphml",
            ExportFormat::Dot => "dot",
        };
        
        if let Some(extension) = path.extension() {
            if extension.to_string_lossy().to_lowercase() != expected_extension {
                return Err(GraphError::Export(format!(
                    "File extension should be .{} for {:?} format",
                    expected_extension,
                    format
                )));
            }
        }
        
        Ok(())
    }
}

impl Default for GraphExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Html,
            include_metadata: true,
            include_styling: true,
            compact_output: false,
            file_path: None,
        }
    }
}
