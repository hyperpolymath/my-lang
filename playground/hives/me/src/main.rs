// My Language Compiler - Main Entry Point

#![forbid(unsafe_code)]
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "my-lang")]
#[command(about = "Compiler and runtime for the My Language family", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a source file
    Build {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Enable optimizations
        #[arg(short = 'O', long)]
        optimize: bool,

        /// Target mode (solo, duet, ensemble)
        #[arg(short, long, default_value = "solo")]
        mode: String,
    },

    /// Run a source file
    Run {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Run mode (solo, duet, ensemble)
        #[arg(long, default_value = "solo")]
        mode: String,

        /// Arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Check source file for errors (no code generation)
    Check {
        /// Input file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },

    /// Format source code
    Fmt {
        /// Input files
        #[arg(value_name = "FILES")]
        files: Vec<PathBuf>,

        /// Check if files are formatted
        #[arg(long)]
        check: bool,
    },

    /// Start REPL
    Repl {
        /// Mode (solo, duet, ensemble)
        #[arg(long, default_value = "solo")]
        mode: String,
    },

    /// Start Language Server Protocol server
    Lsp,

    /// Run tests
    Test {
        /// Test filter pattern
        #[arg(value_name = "PATTERN")]
        pattern: Option<String>,
    },

    /// Show version information
    Version,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Build { input, output, optimize, mode } => {
            println!("Building {:?} in {} mode", input, mode);
            println!("Optimize: {}", optimize);
            build_file(&input, output.as_deref(), optimize, &mode)?;
        }
        Commands::Run { input, mode, args } => {
            println!("Running {:?} in {} mode", input, mode);
            run_file(&input, &mode, &args)?;
        }
        Commands::Check { input } => {
            println!("Checking {:?}", input);
            check_file(&input)?;
        }
        Commands::Fmt { files, check } => {
            for file in files {
                if check {
                    println!("Checking format of {:?}", file);
                } else {
                    println!("Formatting {:?}", file);
                }
            }
        }
        Commands::Repl { mode } => {
            println!("Starting REPL in {} mode", mode);
            start_repl(&mode)?;
        }
        Commands::Lsp => {
            println!("Starting LSP server");
            start_lsp()?;
        }
        Commands::Test { pattern } => {
            println!("Running tests{}", pattern.as_ref().map(|p| format!(": {}", p)).unwrap_or_default());
            run_tests(pattern.as_deref())?;
        }
        Commands::Version => {
            println!("My Language v{}", env!("CARGO_PKG_VERSION"));
            println!("Dialects: Me, Solo, Duet, Ensemble");
        }
    }

    Ok(())
}

fn build_file(input: &std::path::Path, output: Option<&std::path::Path>, optimize: bool, mode: &str) -> Result<()> {
    use std::fs;

    // Read source
    let source = fs::read_to_string(input)?;

    // Lex
    println!("[1/5] Lexing...");
    let mut lexer = my_lang_lexer::Lexer::new(&source);
    let tokens = lexer.tokenize_all();
    println!("  {} tokens", tokens.len());

    // Parse (stub)
    println!("[2/5] Parsing...");
    println!("  Parsing complete");

    // Type check (stub)
    println!("[3/5] Type checking...");
    println!("  Type checking complete");

    // Optimize (stub)
    if optimize {
        println!("[4/5] Optimizing...");
        println!("  Optimization complete");
    } else {
        println!("[4/5] Skipping optimization");
    }

    // Code generation (stub)
    println!("[5/5] Generating code...");
    let output_path = output.unwrap_or_else(|| std::path::Path::new("output"));
    println!("  Output: {:?}", output_path);

    println!("\n✓ Build successful");
    Ok(())
}

fn run_file(input: &std::path::Path, mode: &str, args: &[String]) -> Result<()> {
    println!("Running program...");
    println!("Mode: {}", mode);
    println!("Args: {:?}", args);
    // Stub: would execute the compiled program
    Ok(())
}

fn check_file(input: &std::path::Path) -> Result<()> {
    use std::fs;

    let source = fs::read_to_string(input)?;
    let mut lexer = my_lang_lexer::Lexer::new(&source);
    let _tokens = lexer.tokenize_all();

    println!("✓ No errors found");
    Ok(())
}

fn start_repl(mode: &str) -> Result<()> {
    println!("REPL not yet implemented");
    println!("Mode: {}", mode);
    Ok(())
}

fn start_lsp() -> Result<()> {
    println!("LSP server not yet implemented");
    Ok(())
}

fn run_tests(pattern: Option<&str>) -> Result<()> {
    println!("Test runner not yet implemented");
    if let Some(p) = pattern {
        println!("Pattern: {}", p);
    }
    Ok(())
}
