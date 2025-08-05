use crate::config::GraphConfig;
use crate::graph_builder::InteractiveGraph;
use crate::error::{GraphError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsOptions {
    pub nodes: VisJsNodeOptions,
    pub edges: VisJsEdgeOptions,
    pub layout: VisJsLayoutOptions,
    pub physics: VisJsPhysicsOptions,
    pub interaction: VisJsInteractionOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsNodeOptions {
    pub shape: String,
    pub size: f64,
    pub font: VisJsFontOptions,
    pub border_width: f64,
    pub shadow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsEdgeOptions {
    pub width: f64,
    pub arrows: VisJsArrowOptions,
    pub smooth: bool,
    pub shadow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsLayoutOptions {
    pub hierarchical: Option<VisJsHierarchicalOptions>,
    pub random_seed: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsHierarchicalOptions {
    pub enabled: bool,
    pub direction: String,
    pub sort_method: String,
    pub node_spacing: f64,
    pub level_separation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsPhysicsOptions {
    pub enabled: bool,
    pub stabilization: VisJsStabilizationOptions,
    pub repulsion: VisJsRepulsionOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsStabilizationOptions {
    pub enabled: bool,
    pub iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsRepulsionOptions {
    pub node_distance: f64,
    pub central_gravity: f64,
    pub spring_length: f64,
    pub spring_constant: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsInteractionOptions {
    pub drag_nodes: bool,
    pub drag_view: bool,
    pub zoom_view: bool,
    pub select_connected_edges: bool,
    pub hover: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsFontOptions {
    pub size: u32,
    pub color: String,
    pub face: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsArrowOptions {
    pub to: VisJsArrowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisJsArrowConfig {
    pub enabled: bool,
    pub scale_factor: f64,
}

pub struct WebInterface {
    config: GraphConfig,
    container_id: String,
}

impl WebInterface {
    pub fn new(container_id: String) -> WebInterface {
        WebInterface {
            config: GraphConfig::default(),
            container_id,
        }
    }

    pub fn set_config(&mut self, config: GraphConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }

    pub fn get_container_id(&self) -> &str {
        &self.container_id
    }

    pub fn prepare_vis_js_data(&self, graph: &InteractiveGraph) -> Result<(String, String, String)> {
        let nodes_data = self.prepare_vis_js_nodes(&graph.nodes)?;
        let edges_data = self.prepare_vis_js_edges(&graph.edges)?;
        let options = self.prepare_vis_js_options(&graph.config)?;
        
        Ok((nodes_data, edges_data, options))
    }

    fn prepare_vis_js_nodes(&self, nodes: &[crate::graph_builder::GraphNode]) -> Result<String> {
        let vis_nodes: Vec<serde_json::Value> = nodes.iter().map(|node| {
            serde_json::json!({
                "id": node.id,
                "label": node.label,
                "color": node.color,
                "shape": node.shape,
                "size": node.size,
                "x": node.x,
                "y": node.y,
                "physics": node.physics,
                "title": format!("Type: {:?}<br/>Confidence: {:.2}", node.node_type, node.metadata.confidence),
                "group": format!("{:?}", node.node_type).to_lowercase()
            })
        }).collect();

        serde_json::to_string(&vis_nodes)
            .map_err(|e| GraphError::WebInterface(format!("Failed to serialize nodes: {}", e)))
    }

    fn prepare_vis_js_edges(&self, edges: &[crate::graph_builder::GraphEdge]) -> Result<String> {
        let vis_edges: Vec<serde_json::Value> = edges.iter().map(|edge| {
            serde_json::json!({
                "id": edge.id,
                "from": edge.from,
                "to": edge.to,
                "label": edge.label,
                "color": edge.color,
                "width": edge.width,
                "arrows": edge.arrows,
                "title": format!("Type: {}<br/>Confidence: {:.2}", edge.metadata.relationship_type, edge.metadata.confidence),
                "smooth": {
                    "type": "continuous"
                }
            })
        }).collect();

        serde_json::to_string(&vis_edges)
            .map_err(|e| GraphError::WebInterface(format!("Failed to serialize edges: {}", e)))
    }

    fn prepare_vis_js_options(&self, config: &GraphConfig) -> Result<String> {
        let options = VisJsOptions {
            nodes: VisJsNodeOptions {
                shape: "dot".to_string(),
                size: 25.0,
                font: VisJsFontOptions {
                    size: 14,
                    color: "#343434".to_string(),
                    face: "arial".to_string(),
                },
                border_width: 2.0,
                shadow: true,
            },
            edges: VisJsEdgeOptions {
                width: 2.0,
                arrows: VisJsArrowOptions {
                    to: VisJsArrowConfig {
                        enabled: true,
                        scale_factor: 1.0,
                    },
                },
                smooth: true,
                shadow: true,
            },
            layout: if config.layout.hierarchical {
                VisJsLayoutOptions {
                    hierarchical: Some(VisJsHierarchicalOptions {
                        enabled: true,
                        direction: "UD".to_string(), // Up-Down
                        sort_method: "directed".to_string(),
                        node_spacing: config.layout.spacing,
                        level_separation: 150.0,
                    }),
                    random_seed: Some(2),
                }
            } else {
                VisJsLayoutOptions {
                    hierarchical: None,
                    random_seed: Some(2),
                }
            },
            physics: VisJsPhysicsOptions {
                enabled: config.physics.enabled,
                stabilization: VisJsStabilizationOptions {
                    enabled: config.physics.stabilization,
                    iterations: 1000,
                },
                repulsion: VisJsRepulsionOptions {
                    node_distance: config.physics.repulsion,
                    central_gravity: 0.1,
                    spring_length: config.physics.spring_length,
                    spring_constant: config.physics.spring_constant,
                },
            },
            interaction: VisJsInteractionOptions {
                drag_nodes: true,
                drag_view: true,
                zoom_view: true,
                select_connected_edges: true,
                hover: true,
            },
        };

        serde_json::to_string(&options)
            .map_err(|e| GraphError::WebInterface(format!("Failed to serialize options: {}", e)))
    }

    pub fn create_html_template(&self, title: &str) -> String {
        format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <script type="text/javascript" src="https://unpkg.com/vis-network/standalone/umd/vis-network.min.js"></script>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 0;
            background-color: #f5f5f5;
            overflow: hidden;
        }}
        
        .header {{
            background-color: #2c3e50;
            color: white;
            padding: 15px 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            z-index: 1000;
            position: relative;
        }}
        
        .main-container {{
            display: flex;
            height: calc(100vh - 70px);
            position: relative;
        }}
        
        .side-panel {{
            width: 300px;
            background-color: white;
            box-shadow: 2px 0 4px rgba(0,0,0,0.1);
            overflow-y: auto;
            transition: transform 0.3s ease;
            z-index: 100;
        }}
        
        .side-panel.collapsed {{
            transform: translateX(-100%);
        }}
        
        .panel-toggle {{
            position: absolute;
            left: 10px;
            top: 50%;
            transform: translateY(-50%);
            background-color: #34495e;
            color: white;
            border: none;
            padding: 10px;
            border-radius: 0 5px 5px 0;
            cursor: pointer;
            z-index: 200;
            transition: all 0.3s ease;
        }}
        
        .panel-toggle.active {{
            background-color: #27ae60;
            left: 310px;
        }}
        
        .panel-toggle.collapsed {{
            background-color: #e74c3c;
            left: 10px;
        }}
        
        .controls {{
            padding: 20px;
        }}
        
        .control-section {{
            margin-bottom: 20px;
            border: 1px solid #e0e0e0;
            border-radius: 8px;
            overflow: hidden;
        }}
        
        .section-header {{
            background-color: #ecf0f1;
            padding: 12px 15px;
            font-weight: bold;
            cursor: pointer;
            border-bottom: 1px solid #e0e0e0;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}
        
        .section-header:hover {{
            background-color: #d5dbdb;
        }}
        
        .section-content {{
            padding: 15px;
            display: none;
        }}
        
        .section-content.expanded {{
            display: block;
        }}
        
        .control-group {{
            margin-bottom: 15px;
            padding: 10px;
            background-color: #f9f9f9;
            border-radius: 4px;
        }}
        
        .control-group label {{
            font-weight: bold;
            margin-bottom: 8px;
            display: block;
        }}
        
        .graph-container {{
            flex: 1;
            background-color: white;
            position: relative;
        }}
        
        #{} {{
            width: 100%;
            height: 100%;
            border: none;
        }}
        
        .info-panel {{
            position: absolute;
            top: 20px;
            right: 20px;
            background-color: white;
            padding: 15px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            max-width: 300px;
            min-width: 250px;
            z-index: 50;
            transition: transform 0.3s ease, opacity 0.3s ease;
        }}
        
        .info-panel.collapsed {{
            transform: translateX(calc(100% + 20px));
            opacity: 0;
        }}
        
        .info-panel h3 {{
            margin-top: 0;
            margin-bottom: 15px;
            font-size: 16px;
            color: #34495e;
            border-bottom: 1px solid #e0e0e0;
            padding-bottom: 8px;
        }}
        
        .info-toggle {{
            position: absolute;
            top: 20px;
            right: 20px;
            background-color: #34495e;
            color: white;
            border: none;
            padding: 12px;
            border-radius: 5px;
            cursor: pointer;
            z-index: 60;
            transition: all 0.3s ease;
            font-size: 16px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.2);
        }}
        
        .info-toggle.panel-open {{
            right: 290px;
            background-color: #27ae60;
        }}
        
        .info-toggle:hover {{
            background-color: #2c3e50;
            transform: scale(1.05);
        }}
        
        .info-toggle.panel-open:hover {{
            background-color: #229954;
            transform: scale(1.05);
        }}
        
        button {{
            background-color: #3498db;
            color: white;
            border: none;
            padding: 8px 16px;
            margin: 3px;
            border-radius: 4px;
            cursor: pointer;
            transition: background-color 0.3s ease;
        }}
        
        button:hover {{
            background-color: #2980b9;
        }}
        
        button.toggle-off {{
            background-color: #e74c3c;
        }}
        
        button.toggle-off:hover {{
            background-color: #c0392b;
        }}
        
        button.toggle-on {{
            background-color: #27ae60;
        }}
        
        button.toggle-on:hover {{
            background-color: #229954;
        }}
        
        select, input {{
            padding: 8px;
            margin: 3px;
            border: 1px solid #ddd;
            border-radius: 4px;
            width: 100%;
            box-sizing: border-box;
        }}
        
        .node-info, .edge-info {{
            background-color: #ecf0f1;
            padding: 10px;
            border-radius: 4px;
            margin-top: 10px;
            display: none;
        }}
        
        .expand-icon {{
            transition: transform 0.3s ease;
        }}
        
        .expand-icon.rotated {{
            transform: rotate(180deg);
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>{}</h1>
        <p>Interactive Entity Relationship Graph Visualizer</p>
    </div>
    
    <div class="main-container">
        <button class="panel-toggle active" onclick="toggleSidePanel()">☰</button>
        
        <div class="side-panel" id="sidePanel">
            <div class="controls">
                <h3>Graph Controls</h3>
                
                <!-- Layout Controls -->
                <div class="control-section">
                    <div class="section-header" onclick="toggleSection('layout')">
                        Layout Controls
                        <span class="expand-icon">▼</span>
                    </div>
                    <div class="section-content expanded" id="layout">
                        <div class="control-group">
                            <label>Layout Type:</label>
                            <button onclick="changeLayout('hierarchical')">Hierarchical</button>
                            <button onclick="changeLayout('force')">Force-Directed</button>
                            <button onclick="changeLayout('circular')">Circular</button>
                        </div>
                    </div>
                </div>
                
                <!-- View Controls -->
                <div class="control-section">
                    <div class="section-header" onclick="toggleSection('view')">
                        View Controls
                        <span class="expand-icon">▼</span>
                    </div>
                    <div class="section-content expanded" id="view">
                        <div class="control-group">
                            <label>Zoom & Position:</label>
                            <button onclick="zoomIn()">Zoom In</button>
                            <button onclick="zoomOut()">Zoom Out</button>
                            <button onclick="fitGraph()">Fit to View</button>
                            <button onclick="centerGraph()">Center Graph</button>
                        </div>
                    </div>
                </div>
                
                <!-- Physics Controls -->
                <div class="control-section">
                    <div class="section-header" onclick="toggleSection('physics')">
                        Physics Controls
                        <span class="expand-icon">▼</span>
                    </div>
                    <div class="section-content expanded" id="physics">
                        <div class="control-group">
                            <label>Physics Simulation:</label>
                            <button id="physicsToggle" class="toggle-on" onclick="togglePhysics()">Physics: ON</button>
                            <button onclick="stabilizeGraph()">Stabilize</button>
                        </div>
                    </div>
                </div>
                
                <!-- Label Controls -->
                <div class="control-section">
                    <div class="section-header" onclick="toggleSection('labels')">
                        Label Controls
                        <span class="expand-icon">▼</span>
                    </div>
                    <div class="section-content" id="labels">
                        <div class="control-group">
                            <label>Label Visibility:</label>
                            <button id="nodeLabelsToggle" class="toggle-on" onclick="toggleNodeLabels()">Node Labels: ON</button>
                            <button id="edgeLabelsToggle" class="toggle-on" onclick="toggleEdgeLabels()">Edge Labels: ON</button>
                        </div>
                    </div>
                </div>
                
                <!-- Filter Controls -->
                <div class="control-section">
                    <div class="section-header" onclick="toggleSection('filters')">
                        Filter Controls
                        <span class="expand-icon">▼</span>
                    </div>
                    <div class="section-content" id="filters">
                        <div class="control-group">
                            <label>Node Type Filter:</label>
                            <select onchange="filterNodes(this.value)">
                                <option value="">Show All Nodes</option>
                                <option value="entity">Entities Only</option>
                                <option value="concept">Concepts Only</option>
                                <option value="attribute">Attributes Only</option>
                            </select>
                        </div>
                    </div>
                </div>
                
                <!-- Export Controls -->
                <div class="control-section">
                    <div class="section-header" onclick="toggleSection('export')">
                        Export Controls
                        <span class="expand-icon">▼</span>
                    </div>
                    <div class="section-content" id="export">
                        <div class="control-group">
                            <label>Export Options:</label>
                            <button onclick="exportGraph('json')">Export JSON</button>
                            <button onclick="exportGraph('png')">Export PNG</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="graph-container">
            <div id="{}"></div>
            
            <button class="info-toggle panel-open" id="infoToggle" onclick="toggleInfoPanel()">ℹ️</button>
            
            <div class="info-panel" id="infoPanel">
                <h3>Information Panel</h3>
                <div id="node-info" class="node-info">
                    <h4>Node Information</h4>
                    <div id="node-details"></div>
                </div>
                <div id="edge-info" class="edge-info">
                    <h4>Edge Information</h4>
                    <div id="edge-details"></div>
                </div>
            </div>
        </div>
    </div>
    
    <script>
        // Global variables
        let currentNetwork = null;
        let originalNodes = null;
        let originalEdges = null;
        let showNodeLabels = true;
        let showEdgeLabels = true;
        let physicsEnabled = true;
        let sidePanelOpen = true;
        let infoPanelOpen = true;
        
        // Side panel and section controls
        function toggleSidePanel() {{
            const panel = document.getElementById('sidePanel');
            const toggle = document.querySelector('.panel-toggle');
            
            sidePanelOpen = !sidePanelOpen;
            
            if (sidePanelOpen) {{
                panel.classList.remove('collapsed');
                toggle.classList.add('active');
                toggle.classList.remove('collapsed');
                toggle.textContent = '☰';
            }} else {{
                panel.classList.add('collapsed');
                toggle.classList.remove('active');
                toggle.classList.add('collapsed');
                toggle.textContent = '►';
            }}
        }}
        
        function toggleInfoPanel() {{
            const panel = document.getElementById('infoPanel');
            const toggle = document.getElementById('infoToggle');
            
            infoPanelOpen = !infoPanelOpen;
            
            if (infoPanelOpen) {{
                panel.classList.remove('collapsed');
                toggle.classList.add('panel-open');
                toggle.textContent = 'ℹ️';
            }} else {{
                panel.classList.add('collapsed');
                toggle.classList.remove('panel-open');
                toggle.textContent = '►';
            }}
        }}
        
        function toggleSection(sectionId) {{
            const content = document.getElementById(sectionId);
            const header = content.previousElementSibling;
            const icon = header.querySelector('.expand-icon');
            
            if (content.classList.contains('expanded')) {{
                content.classList.remove('expanded');
                icon.classList.remove('rotated');
            }} else {{
                content.classList.add('expanded');
                icon.classList.add('rotated');
            }}
        }}
        
        function updateToggleButton(buttonId, isOn, onText, offText) {{
            const button = document.getElementById(buttonId);
            if (isOn) {{
                button.className = 'toggle-on';
                button.textContent = onText;
            }} else {{
                button.className = 'toggle-off';
                button.textContent = offText;
            }}
        }}
        
        // Layout change function
        function changeLayout(layoutType) {{
            if (currentNetwork) {{
                console.log('Changing layout to:', layoutType);
                let layoutOptions = {{}};
                
                switch(layoutType) {{
                    case 'hierarchical':
                        layoutOptions = {{
                            hierarchical: {{
                                enabled: true,
                                direction: 'UD',
                                sortMethod: 'directed',
                                nodeSpacing: 200,
                                levelSeparation: 150
                            }}
                        }};
                        break;
                    case 'force':
                        layoutOptions = {{
                            hierarchical: {{ enabled: false }},
                            randomSeed: Math.floor(Math.random() * 1000)
                        }};
                        break;
                    case 'circular':
                        // Implement circular layout
                        const nodes = currentNetwork.body.data.nodes.get();
                        const nodePositions = {{}};
                        const centerX = 0, centerY = 0;
                        const radius = Math.max(200, nodes.length * 20);
                        
                        nodes.forEach((node, index) => {{
                            const angle = (2 * Math.PI * index) / nodes.length;
                            nodePositions[node.id] = {{
                                x: centerX + radius * Math.cos(angle),
                                y: centerY + radius * Math.sin(angle)
                            }};
                        }});
                        
                        currentNetwork.setData({{
                            nodes: nodes.map(n => ({{ ...n, ...nodePositions[n.id] }})),
                            edges: currentNetwork.body.data.edges.get()
                        }});
                        return;
                }}
                
                currentNetwork.setOptions({{ layout: layoutOptions }});
                currentNetwork.stabilize();
            }}
        }}
        
        // View control functions
        function zoomIn() {{
            if (currentNetwork) {{
                const scale = currentNetwork.getScale();
                currentNetwork.moveTo({{ scale: scale * 1.2 }});
            }}
        }}
        
        function zoomOut() {{
            if (currentNetwork) {{
                const scale = currentNetwork.getScale();
                currentNetwork.moveTo({{ scale: scale * 0.8 }});
            }}
        }}
        
        function fitGraph() {{
            if (currentNetwork) {{
                currentNetwork.fit({{ animation: true }});
            }}
        }}
        
        function centerGraph() {{
            if (currentNetwork) {{
                currentNetwork.moveTo({{ position: {{ x: 0, y: 0 }} }});
            }}
        }}
        
        // Physics control functions
        function togglePhysics() {{
            if (currentNetwork) {{
                physicsEnabled = !physicsEnabled;
                const physicsOptions = {{
                    enabled: physicsEnabled,
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
                }};
                currentNetwork.setOptions({{ physics: physicsOptions }});
                updateToggleButton('physicsToggle', physicsEnabled, 'Physics: ON', 'Physics: OFF');
                console.log('Physics:', physicsEnabled ? 'enabled' : 'disabled');
            }}
        }}
        
        function stabilizeGraph() {{
            if (currentNetwork) {{
                currentNetwork.stabilize();
                console.log('Graph stabilization initiated');
                // Optional: Add a timeout to show when stabilization is complete
                currentNetwork.once('stabilizationIterationsDone', function() {{
                    console.log('Graph stabilization completed');
                }});
            }}
        }}
        
        // Label control functions
        function toggleNodeLabels() {{
            if (currentNetwork && originalNodes) {{
                showNodeLabels = !showNodeLabels;
                const nodes = originalNodes.map(node => ({{
                    ...node,
                    label: showNodeLabels ? node.originalLabel || node.label : ''
                }}));
                
                currentNetwork.setData({{
                    nodes: nodes,
                    edges: currentNetwork.body.data.edges.get()
                }});
                updateToggleButton('nodeLabelsToggle', showNodeLabels, 'Node Labels: ON', 'Node Labels: OFF');
                console.log('Node labels:', showNodeLabels ? 'shown' : 'hidden');
            }}
        }}
        
        function toggleEdgeLabels() {{
            if (currentNetwork && originalEdges) {{
                showEdgeLabels = !showEdgeLabels;
                const edges = originalEdges.map(edge => ({{
                    ...edge,
                    label: showEdgeLabels ? edge.originalLabel || edge.label : ''
                }}));
                
                currentNetwork.setData({{
                    nodes: currentNetwork.body.data.nodes.get(),
                    edges: edges
                }});
                updateToggleButton('edgeLabelsToggle', showEdgeLabels, 'Edge Labels: ON', 'Edge Labels: OFF');
                console.log('Edge labels:', showEdgeLabels ? 'shown' : 'hidden');
            }}
        }}
        
        // Node filtering function
        function filterNodes(nodeType) {{
            if (currentNetwork && originalNodes) {{
                console.log('Filtering nodes by type:', nodeType);
                
                let filteredNodes = originalNodes;
                let filteredEdges = originalEdges;
                
                if (nodeType) {{
                    filteredNodes = originalNodes.filter(node => 
                        node.group === nodeType || 
                        node.node_type === nodeType || 
                        node.type === nodeType
                    );
                    
                    const nodeIds = new Set(filteredNodes.map(n => n.id));
                    filteredEdges = originalEdges.filter(edge => 
                        nodeIds.has(edge.from) && nodeIds.has(edge.to)
                    );
                }}
                
                currentNetwork.setData({{
                    nodes: filteredNodes,
                    edges: filteredEdges
                }});
            }}
        }}
        
        // Export functions
        function exportGraph(format) {{
            console.log('Exporting graph as:', format);
            
            if (format === 'json') {{
                const graphData = {{
                    nodes: currentNetwork.body.data.nodes.get(),
                    edges: currentNetwork.body.data.edges.get(),
                    config: window.graphData.config
                }};
                
                const dataStr = JSON.stringify(graphData, null, 2);
                const dataBlob = new Blob([dataStr], {{ type: 'application/json' }});
                const url = URL.createObjectURL(dataBlob);
                
                const link = document.createElement('a');
                link.href = url;
                link.download = 'graph_export.json';
                link.click();
                
                URL.revokeObjectURL(url);
            }} else if (format === 'png') {{
                // Note: PNG export requires additional vis.js configuration
                console.log('PNG export not implemented in this version');
                alert('PNG export requires server-side rendering. Use browser screenshot instead.');
            }}
        }}
        
        // Node and edge selection handlers
        function onNodeSelected(nodeId) {{
            console.log('Node selected:', nodeId);
            const nodeData = currentNetwork.body.data.nodes.get(nodeId);
            
            document.getElementById('node-info').style.display = 'block';
            document.getElementById('edge-info').style.display = 'none';
            
            if (nodeData) {{
                document.getElementById('node-details').innerHTML = `
                    <strong>ID:</strong> ${{nodeData.id}}<br/>
                    <strong>Label:</strong> ${{nodeData.label}}<br/>
                    <strong>Type:</strong> ${{nodeData.node_type || nodeData.group || 'Unknown'}}<br/>
                    <strong>Confidence:</strong> ${{nodeData.confidence || 'N/A'}}
                `;
            }}
        }}
        
        function onEdgeSelected(edgeId) {{
            console.log('Edge selected:', edgeId);
            const edgeData = currentNetwork.body.data.edges.get(edgeId);
            
            document.getElementById('edge-info').style.display = 'block';
            document.getElementById('node-info').style.display = 'none';
            
            if (edgeData) {{
                document.getElementById('edge-details').innerHTML = `
                    <strong>ID:</strong> ${{edgeData.id}}<br/>
                    <strong>From:</strong> ${{edgeData.from}}<br/>
                    <strong>To:</strong> ${{edgeData.to}}<br/>
                    <strong>Label:</strong> ${{edgeData.label}}<br/>
                    <strong>Type:</strong> ${{edgeData.relationship_type || 'Unknown'}}
                `;
            }}
        }}
    </script>
</body>
</html>
        "#, title, self.container_id, title, self.container_id)
    }
}
