use clap::{Parser, Subcommand};
use msg_net::{
    config::GraphConfig,
    entity_extractor::EntityExtractor,
    export::{ExportFormat, ExportOptions, GraphExporter},
    graph_builder::GraphBuilder,
    text_processor::{SourceType, TextProcessor},
    Result,
};
use std::fs;

#[derive(Parser)]
#[command(name = "msg_net")]
#[command(about = "\t Entity Relationship Graph Visualizer - Convert text into interactive network graphs")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process text and generate an interactive graph
    Generate {
        /// Input text file path
        #[arg(short, long)]
        input: String,
        
        /// Output file path (format determined by extension)
        #[arg(short, long)]
        output: String,
        
        /// Source type of the input text
        #[arg(short, long, default_value = "document")]
        source_type: String,
        
        /// Configuration file path (JSON)
        #[arg(short, long)]
        config: Option<String>,
        
        /// Export format
        #[arg(short, long, default_value = "html")]
        format: String,
        
        /// Include metadata in export
        #[arg(long)]
        include_metadata: bool,
        
        /// Use LLM for enhanced extraction
        #[arg(long)]
        use_llm: bool,
        
        /// Use deep analysis with LLM for comprehensive relationship extraction
        #[arg(long)]
        deep_analysis: bool,
        
        /// LLM model to use (e.g., llama3.2)
        #[arg(long, default_value = "llama3.2")]
        llm_model: String,
        
        /// LLM endpoint URL
        #[arg(long, default_value = "http://localhost:11434/api/generate")]
        llm_endpoint: String,
    },
    
    /// Validate and process text without generating output
    Analyze {
        /// Input text file path
        #[arg(short, long)]
        input: String,
        
        /// Show detailed analysis
        #[arg(short, long)]
        verbose: bool,
        
        /// Configuration file path (JSON)
        #[arg(short, long)]
        config: Option<String>,
    },
    
    /// Generate a sample configuration file
    Config {
        /// Output path for the configuration file
        #[arg(short, long, default_value = "graph_config.json")]
        output: String,
    },
    
    /// Show example usage and sample text
    Example {
        /// Generate example text file
        #[arg(short, long)]
        generate_text: bool,
        
        /// Generate AI story using Ollama (requires --word-count)
        #[arg(long)]
        generate_ai_story: bool,
        
        /// Number of words for AI-generated story
        #[arg(long, default_value = "200")]
        word_count: usize,
        
        /// LLM model to use for AI story generation
        #[arg(long, default_value = "llama3.2")]
        llm_model: String,
        
        /// LLM endpoint URL for AI story generation
        #[arg(long, default_value = "http://localhost:11434/api/generate")]
        llm_endpoint: String,
        
        /// Output path for example text
        #[arg(short, long, default_value = "example_text.txt")]
        output: String,
    },
    
    /// Show comprehensive usage examples and command samples
    BigHelp,
}


// use colored::Colorize;

mod toml_extract; // Extract and print the version information according to the toml file

// Function to display the banner
fn show_banner() {
    let banner = String::from(
        "\n
\t 
\t ███╗   ███╗   ███████╗    ██████╗    
\t ████╗ ████║   ██╔════╝   ██╔════╝    
\t ██╔████╔██║   ███████╗   ██║  ███╗   
\t ██║╚██╔╝██║   ╚════██║   ██║   ██║   
\t ██║ ╚═╝ ██║   ███████║   ╚██████╔╝   
\t ╚═╝     ╚═╝   ╚══════╝    ╚═════╝    
\t 
\t ███╗   ██╗   ███████╗   ████████╗   
\t ████╗  ██║   ██╔════╝   ╚══██╔══╝   
\t ██╔██╗ ██║   █████╗        ██║      
\t ██║╚██╗██║   ██╔══╝        ██║      
\t ██║ ╚████║   ███████╗      ██║      
\t ╚═╝  ╚═══╝   ╚══════╝      ╚═╝      

",
    );

    // Print the banner in purple color
    toml_extract::colour_print(&banner, "cyan");
}



#[tokio::main]
async fn main() -> Result<()> {
    // Show the banner
    show_banner();

    // Display version information from the toml file
    toml_extract::main();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            input,
            output,
            source_type,
            config,
            format,
            include_metadata,
            use_llm,
            deep_analysis,
            llm_model,
            llm_endpoint,
        } => {
            generate_graph(
                &input,
                &output,
                &source_type,
                config.as_deref(),
                &format,
                include_metadata,
                use_llm,
                deep_analysis,
                &llm_model,
                &llm_endpoint,
            )
            .await
        }
        Commands::Analyze {
            input,
            verbose,
            config,
        } => analyze_text(&input, verbose, config.as_deref()).await,
        Commands::Config { output } => generate_config(&output),
        Commands::Example {
            generate_text,
            generate_ai_story,
            word_count,
            llm_model,
            llm_endpoint,
            output,
        } => {
            if generate_ai_story {
                generate_ai_story_text(&output, word_count, &llm_model, &llm_endpoint).await
            } else if generate_text {
                generate_example_text(&output)
            } else {
                show_usage_examples()
            }
        }
        Commands::BigHelp => show_comprehensive_help(),
    }
}

async fn generate_graph(
    input_path: &str,
    output_path: &str,
    source_type: &str,
    config_path: Option<&str>,
    format: &str,
    include_metadata: bool,
    use_llm: bool,
    deep_analysis: bool,
    llm_model: &str,
    llm_endpoint: &str,
) -> Result<()> {
    println!("🚀 Starting Entity Relationship Graph generation...");
    
    // Load and validate input
    let text = fs::read_to_string(input_path)
        .map_err(|e| msg_net::error::GraphError::Io(e))?;
    
    if text.trim().is_empty() {
        return Err(msg_net::error::GraphError::TextProcessing(
            "Input file is empty".to_string(),
        ));
    }

    println!("📖 Loaded text from: {} ({} characters)", input_path, text.len());

    // Load configuration
    let mut config = if let Some(config_path) = config_path {
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| msg_net::error::GraphError::Io(e))?;
        serde_json::from_str::<GraphConfig>(&config_content)
            .map_err(|e| msg_net::error::GraphError::Json(e))?
    } else {
        GraphConfig::default()
    };

    // Override config with CLI options
    if use_llm {
        config.extraction.use_llm = true;
        config.extraction.llm_model = llm_model.to_string();
        config.extraction.llm_endpoint = llm_endpoint.to_string();
    }

    // Parse source type
    let source_type = match source_type.to_lowercase().as_str() {
        "chat" | "chatmessage" => SourceType::ChatMessage,
        "document" | "doc" => SourceType::Document,
        "email" => SourceType::Email,
        "article" => SourceType::Article,
        _ => SourceType::Unknown,
    };

    // Process text
    println!("🔍 Processing text...");
    let processor = TextProcessor::new()?;
    let processed_text = processor.process_text(&text, source_type)?;
    
    println!(
        "📊 Text processed: {} words, {} sentences",
        processed_text.metadata.word_count,
        processed_text.metadata.sentence_count
    );

    // Extract entities, relationships, and concepts
    println!("🧠 Extracting entities and relationships...");
    let extractor = EntityExtractor::new(config.extraction.clone())?;
    let extraction_result = if deep_analysis {
        extractor.extract_with_deep_analysis(&processed_text).await?
    } else {
        extractor.extract_from_text(&processed_text).await?
    };
    
    println!(
        "✨ Extracted: {} entities, {} relationships, {} concepts",
        extraction_result.metadata.total_entities,
        extraction_result.metadata.total_relationships,
        extraction_result.metadata.total_concepts
    );

    // Build graph
    println!("🎯 Building interactive graph...");
    let graph_builder = GraphBuilder::new(config);
    let mut graph = graph_builder.build_graph(&extraction_result, &text)?;
    
    // Apply layout
    graph_builder.apply_layout(&mut graph)?;
    
    println!("📈 Graph built: {} nodes, {} edges", graph.nodes.len(), graph.edges.len());

    // Export graph
    println!("💾 Exporting graph...");
    let export_format = match format.to_lowercase().as_str() {
        "html" => ExportFormat::Html,
        "json" => ExportFormat::Json,
        "csv" => ExportFormat::Csv,
        "graphml" => ExportFormat::GraphML,
        "dot" => ExportFormat::Dot,
        _ => return Err(msg_net::error::GraphError::Export(
            format!("Unsupported export format: {}", format)
        )),
    };

    let export_options = ExportOptions {
        format: export_format,
        include_metadata,
        include_styling: true,
        compact_output: false,
        file_path: Some(output_path.to_string()),
    };

    let exporter = GraphExporter::new();
    GraphExporter::validate_export_path(output_path, &export_options.format)?;
    let export_result = exporter.export_graph(&graph, &export_options)?;

    if export_result.success {
        let actual_path = export_result.file_path.as_deref().unwrap_or(output_path);
        println!("✅ Graph exported successfully to: {}", actual_path);
        if let Some(file_size) = export_result.metadata.file_size_bytes {
            println!("📦 File size: {} bytes", file_size);
        }
        
        if format == "html" {
            println!("🌐 Open the HTML file in your web browser to view the interactive graph!");
        }
    } else {
        if let Some(error) = export_result.error_message {
            return Err(msg_net::error::GraphError::Export(error));
        }
    }

    Ok(())
}

async fn analyze_text(
    input_path: &str,
    verbose: bool,
    config_path: Option<&str>,
) -> Result<()> {
    println!("🔍 Analyzing text file: {}", input_path);

    // Load text
    let text = fs::read_to_string(input_path)
        .map_err(|e| msg_net::error::GraphError::Io(e))?;
    
    if text.trim().is_empty() {
        return Err(msg_net::error::GraphError::TextProcessing(
            "Input file is empty".to_string(),
        ));
    }

    // Load configuration
    let config = if let Some(config_path) = config_path {
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| msg_net::error::GraphError::Io(e))?;
        serde_json::from_str::<GraphConfig>(&config_content)
            .map_err(|e| msg_net::error::GraphError::Json(e))?
    } else {
        GraphConfig::default()
    };

    // Process text
    let processor = TextProcessor::new()?;
    let processed_text = processor.process_text(&text, SourceType::Document)?;

    // Basic analysis
    println!("\n📊 TEXT ANALYSIS RESULTS");
    println!("========================");
    println!("Original length: {} characters", text.len());
    println!("Word count: {}", processed_text.metadata.word_count);
    println!("Sentence count: {}", processed_text.metadata.sentence_count);
    println!("Detected language: {}", processed_text.metadata.language);
    println!("Source type: {:?}", processed_text.metadata.source_type);

    if verbose {
        println!("\n🔍 DETAILED ANALYSIS");
        println!("====================");
        
        // Extract key phrases
        let key_phrases = processor.extract_key_phrases(&processed_text.cleaned_text)?;
        println!("Key phrases found: {}", key_phrases.len());
        for (i, phrase) in key_phrases.iter().take(10).enumerate() {
            println!("  {}. {}", i + 1, phrase);
        }
        
        // Preview entities extraction
        let extractor = EntityExtractor::new(config.extraction.clone())?;
        let extraction_result = extractor.extract_from_text(&processed_text).await?;
        
        println!("\n🧠 ENTITY EXTRACTION PREVIEW");
        println!("============================");
        println!("Entities found: {}", extraction_result.entities.len());
        for (i, entity) in extraction_result.entities.iter().take(5).enumerate() {
            println!("  {}. {} (Type: {:?}, Confidence: {:.2})", 
                     i + 1, entity.name, entity.entity_type, entity.confidence);
        }
        
        println!("Relationships found: {}", extraction_result.relationships.len());
        for (i, rel) in extraction_result.relationships.iter().take(5).enumerate() {
            println!("  {}. {}", i + 1, rel.label);
        }
        
        println!("Concepts found: {}", extraction_result.concepts.len());
        for (i, concept) in extraction_result.concepts.iter().take(5).enumerate() {
            println!("  {}. {}", i + 1, concept.name);
        }
    }

    println!("\n✅ Analysis complete!");
    
    Ok(())
}

fn generate_config(output_path: &str) -> Result<()> {
    println!("📄 Generating sample configuration file...");
    
    let config = GraphConfig::default();
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| msg_net::error::GraphError::Json(e))?;
    
    fs::write(output_path, config_json)
        .map_err(|e| msg_net::error::GraphError::Io(e))?;
    
    println!("✅ Configuration file created: {}", output_path);
    println!("📝 You can edit this file to customize graph appearance and extraction settings.");
    
    Ok(())
}

fn generate_example_text(output_path: &str) -> Result<()> {
    let example_text = r#"
Alice is a software engineer who works at TechCorp. She is responsible for developing the main application that the company uses for customer relationship management. The application has several important features including user authentication, data visualization, and report generation.

Bob, who is Alice's colleague, manages the database system that stores all the customer information. The database system is connected to the main application through a secure API. This API ensures that data flows efficiently between different components of the system.

The customer relationship management system helps the company track interactions with clients. Each client has a unique profile that contains their contact information, purchase history, and communication preferences. The system also generates automated reports that help the sales team understand customer behavior patterns.

TechCorp uses advanced analytics to process the customer data. The analytics module identifies trends and patterns that can help improve customer satisfaction. These insights are shared with the marketing team to develop targeted campaigns.

The development team, led by Carol, continuously improves the system by adding new features and fixing bugs. They use agile methodology to manage their development process. Regular meetings are held to discuss progress and plan future enhancements.
"#;

    fs::write(output_path, example_text.trim())
        .map_err(|e| msg_net::error::GraphError::Io(e))?;
    
    println!("✅ Example text file created: {}", output_path);
    println!("📝 You can use this file to test the graph generation:");
    println!("   msg_net generate -i {} -o example_graph.html", output_path);
    
    Ok(())
}

async fn generate_ai_story_text(
    output_path: &str,
    word_count: usize,
    llm_model: &str,
    llm_endpoint: &str,
) -> Result<()> {
    use serde::{Deserialize, Serialize};

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

    println!("🤖 Generating AI story with {} words using {}...", word_count, llm_model);
    
    let prompt = format!(
        "Write a short story of approximately {} words that includes several characters, locations, and organizations. \
        The story should have clear relationships between entities (people, places, companies) that would be good for \
        creating an entity relationship graph. Include names of people, places, and organizations. \
        Make it interesting and suitable for network analysis. Only return the story text, no additional commentary.",
        word_count
    );

    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: llm_model.to_string(),
        prompt,
        stream: false,
    };

    println!("📡 Calling Ollama API...");
    let response = client
        .post(llm_endpoint)
        .json(&request)
        .send()
        .await
        .map_err(|e| msg_net::error::GraphError::EntityExtraction(format!("Ollama request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(msg_net::error::GraphError::EntityExtraction(format!(
            "Ollama API returned error status: {}",
            response.status()
        )));
    }

    let ollama_response: OllamaResponse = response
        .json()
        .await
        .map_err(|e| msg_net::error::GraphError::EntityExtraction(format!("Failed to parse Ollama response: {}", e)))?;

    let story = ollama_response.response.trim();
    
    // Count words in the generated story
    let actual_words = story.split_whitespace().count();
    
    fs::write(output_path, story)
        .map_err(|e| msg_net::error::GraphError::Io(e))?;
    
    println!("✅ AI-generated story created: {}", output_path);
    println!("📊 Generated {} words (requested: {})", actual_words, word_count);
    println!("📝 You can use this file to test the graph generation:");
    println!("   msg_net generate -i {} -o ai_story_graph.html", output_path);
    
    Ok(())
}

fn show_usage_examples() -> Result<()> {
    println!("🚀 MSG_NET - Entity Relationship Graph Visualizer");
    println!("=================================================");
    println!();
    
    println!("📖 EXAMPLES:");
    println!();
    
    println!("1. Generate an interactive HTML graph from text:");
    println!("   msg_net generate -i input.txt -o graph.html");
    println!();
    
    println!("2. Generate a JSON export with metadata:");
    println!("   msg_net generate -i input.txt -o graph.json -f json --include-metadata");
    println!();
    
    println!("3. Use LLM for enhanced extraction:");
    println!("   msg_net generate -i input.txt -o graph.html --use-llm --llm-model llama3.2");
    println!();
    
    println!("4. Analyze text without generating output:");
    println!("   msg_net analyze -i input.txt --verbose");
    println!();
    
    println!("5. Create a custom configuration file:");
    println!("   msg_net config -o my_config.json");
    println!();
    
    println!("6. Generate example text for testing:");
    println!("   msg_net example --generate-text -o sample.txt");
    println!();
    
    println!("7. Generate AI story with custom word count:");
    println!("   msg_net example --generate-ai-story --word-count 300 -o ai_story.txt");
    println!();
    
    println!("📋 SUPPORTED FORMATS:");
    println!("  • HTML (interactive graph with vis.js)");
    println!("  • JSON (structured data)");
    println!("  • CSV (tabular format)");
    println!("  • GraphML (XML-based graph format)");
    println!("  • DOT (Graphviz format)");
    println!();
    
    println!("🔧 FEATURES:");
    println!("  • Entity extraction (people, places, organizations)");
    println!("  • Relationship detection between entities");
    println!("  • Concept identification and linking");
    println!("  • Interactive web visualization");
    println!("  • Multiple export formats");
    println!("  • Configurable appearance and behavior");
    println!("  • Optional LLM integration for enhanced extraction");
    println!();
    
    Ok(())
}

fn show_comprehensive_help() -> Result<()> {
    println!("🎯 MSG_NET - Comprehensive Usage Guide");
    println!("======================================");
    println!();
    
    println!("🚀 QUICK START COMMANDS:");
    println!("-------------------------");
    println!("1. Basic graph generation:");
    println!("   cargo run -- generate -i sample.txt -o graph.html");
    println!();
    
    println!("2. Create sample data first:");
    println!("   cargo run -- example --generate-text -o sample.txt");
    println!("   cargo run -- generate -i sample.txt -o my_graph.html");
    println!();
    
    println!("2b. Create AI-generated sample data:");
    println!("   cargo run -- example --generate-ai-story --word-count 250 -o ai_sample.txt");
    println!("   cargo run -- generate -i ai_sample.txt -o ai_graph.html");
    println!();
    
    println!("3. Generate with different formats:");
    println!("   cargo run -- generate -i sample.txt -o data.json -f json");
    println!("   cargo run -- generate -i sample.txt -o data.csv -f csv");
    println!("   cargo run -- generate -i sample.txt -o graph.graphml -f graphml");
    println!("   cargo run -- generate -i sample.txt -o graph.dot -f dot");
    println!();
    
    println!("🧠 ADVANCED ANALYSIS:");
    println!("----------------------");
    println!("4. Use LLM for enhanced extraction:");
    println!("   cargo run -- generate -i document.txt -o enhanced.html --use-llm");
    println!();
    
    println!("5. Deep analysis with comprehensive relationship extraction:");
    println!("   cargo run -- generate -i document.txt -o deep.html --use-llm --deep-analysis");
    println!();
    
    println!("6. Custom LLM configuration:");
    println!("   cargo run -- generate -i text.txt -o graph.html --use-llm \\");
    println!("     --llm-model llama3.2 --llm-endpoint http://localhost:11434/api/generate");
    println!();
    
    println!("⚙️  CONFIGURATION & ANALYSIS:");
    println!("------------------------------");
    println!("7. Create custom configuration:");
    println!("   cargo run -- config -o my_config.json");
    println!("   # Edit my_config.json to customize colors, shapes, extraction patterns");
    println!("   cargo run -- generate -i text.txt -o graph.html -c my_config.json");
    println!();
    
    println!("8. Analyze text without generating output:");
    println!("   cargo run -- analyze -i document.txt --verbose");
    println!("   cargo run -- analyze -i document.txt -c my_config.json");
    println!();
    
    println!("📊 WORKFLOW EXAMPLES:");
    println!("----------------------");
    println!("9. Complete workflow for new project:");
    println!("   # Generate sample text");
    println!("   cargo run -- example --generate-text -o project_docs.txt");
    println!("   # Or generate AI story");
    println!("   cargo run -- example --generate-ai-story --word-count 400 -o ai_project_docs.txt");
    println!("   # Create custom config");
    println!("   cargo run -- config -o project_config.json");
    println!("   # Analyze first");
    println!("   cargo run -- analyze -i project_docs.txt -c project_config.json --verbose");
    println!("   # Generate final graph");
    println!("   cargo run -- generate -i project_docs.txt -o project_graph.html -c project_config.json");
    println!();
    
    println!("10. Batch processing with different source types:");
    println!("    cargo run -- generate -i chat_log.txt -o chat_graph.html -s chat");
    println!("    cargo run -- generate -i email_thread.txt -o email_graph.html -s email");
    println!("    cargo run -- generate -i research_paper.txt -o paper_graph.html -s article");
    println!("    cargo run -- generate -i technical_doc.txt -o doc_graph.html -s document");
    println!();
    
    println!("🔬 DEEP ANALYSIS EXAMPLES:");
    println!("---------------------------");
    println!("11. Compare standard vs deep analysis:");
    println!("    # Standard analysis");
    println!("    cargo run -- generate -i complex_doc.txt -o standard.html --use-llm");
    println!("    # Deep analysis");
    println!("    cargo run -- generate -i complex_doc.txt -o deep.html --use-llm --deep-analysis");
    println!();
    
    println!("12. Export deep analysis results in multiple formats:");
    println!("    cargo run -- generate -i document.txt -o results.html --use-llm --deep-analysis");
    println!("    cargo run -- generate -i document.txt -o results.json -f json --use-llm --deep-analysis --include-metadata");
    println!();
    
    println!("📁 FILE ORGANIZATION:");
    println!("----------------------");
    println!("   All output files are automatically organized in the '0_networks/' directory");
    println!("   Files are automatically numbered if they already exist (file.html -> file_01.html)");
    println!();
    
    println!("🎮 INTERACTIVE FEATURES:");
    println!("-------------------------");
    println!("   The generated HTML files include:");
    println!("   • Collapsible side panel with controls");
    println!("   • Layout switching (Hierarchical, Force-Directed, Circular)");
    println!("   • Zoom controls (In, Out, Fit to View, Center)");
    println!("   • Physics simulation toggle");
    println!("   • Label visibility controls (Node/Edge labels)");
    println!("   • Node type filtering (Entities, Concepts, Attributes)");
    println!("   • Export functionality (JSON, PNG)");
    println!("   • Node/Edge selection with detailed information");
    println!();
    
    println!("🛠️  TROUBLESHOOTING:");
    println!("---------------------");
    println!("   If you encounter issues:");
    println!("   1. Check input file exists and is readable");
    println!("   2. For LLM features, ensure Ollama is running: 'ollama serve'");
    println!("   3. For deep analysis, ensure LLM model is available: 'ollama pull llama3.2'");
    println!("   4. Use 'cargo clean && cargo build' if compilation fails");
    println!();
    
    println!("📖 MORE HELP:");
    println!("--------------");
    println!("   • Use 'cargo run -- help' for basic CLI help");
    println!("   • Use 'cargo run -- generate --help' for generate command options");
    println!("   • Use 'cargo run -- analyze --help' for analyze command options");
    println!("   • Check README.md for comprehensive documentation");
    println!();
    
    println!("🎯 RECOMMENDED STARTING COMMANDS:");
    println!("----------------------------------");
    println!("   For first-time users:");
    println!("   1. cargo run -- example --generate-text -o sample.txt");
    println!("   2. cargo run -- generate -i sample.txt -o my_first_graph.html");
    println!("   3. Open 0_networks/my_first_graph.html in your browser");
    println!();
    
    println!("   For users with Ollama setup:");
    println!("   1. cargo run -- example --generate-ai-story --word-count 300 -o ai_sample.txt");
    println!("   2. cargo run -- generate -i ai_sample.txt -o ai_graph.html --use-llm --deep-analysis");
    println!("   3. Open 0_networks/ai_graph.html in your browser");
    println!();
    
    println!("   For advanced users:");
    println!("   1. cargo run -- config -o advanced_config.json");
    println!("   2. cargo run -- generate -i your_document.txt -o advanced.html --use-llm --deep-analysis -c advanced_config.json");
    println!();
    
    Ok(())
}
