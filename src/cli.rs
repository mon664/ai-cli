use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ai-cli",
    version = "0.1.0",
    author = "AI CLI Contributors",
    about = "AI-powered CLI Git Assistant - Generate conventional commits and explain changes",
    long_about = "AI CLI is an intelligent command-line tool that helps developers write better commit messages and understand code changes using AI. It supports both local and remote AI models for privacy and flexibility."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a conventional commit message based on staged changes
    Commit {
        /// Provide extra context or instructions to the AI
        #[arg(short, long)]
        pub message: Option<String>,

        /// Automatically stage all changes (git add -A) before committing
        #[arg(short, long)]
        pub all: bool,

        /// Use specific AI model (local: ollama, remote: openai, anthropic)
        #[arg(short, long, default_value = "local")]
        pub model: String,

        /// Force commit without confirmation (use with caution)
        #[arg(short, long)]
        pub yes: bool,
    },

    /// Explain the staged (or specific commit) changes in natural language
    Explain {
        /// Target a specific commit hash instead of staged changes
        #[arg(long)]
        pub hash: Option<String>,

        /// Use specific AI model
        #[arg(short, long, default_value = "local")]
        pub model: String,

        /// Output format (text, markdown, json)
        #[arg(short, long, default_value = "text")]
        pub format: String,

        /// Include detailed line-by-line analysis
        #[arg(long)]
        pub detailed: bool,
    },

    /// Initialize AI CLI configuration
    Init {
        /// Set default AI model
        #[arg(short, long)]
        pub model: Option<String>,

        /// Set OpenAI API key
        #[arg(long)]
        pub openai_key: Option<String>,

        /// Set Anthropic API key
        #[arg(long)]
        pub anthropic_key: Option<String>,

        /// Ollama server URL
        #[arg(long, default_value = "http://localhost:11434")]
        pub ollama_url: String,
    },

    /// Show current configuration
    Config {
        /// Show all configuration details
        #[arg(short, long)]
        pub verbose: bool,
    },
}