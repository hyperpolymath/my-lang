// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! My Language Debugger
//!
//! Interactive debugger with breakpoints, step-through, and variable inspection.

#![forbid(unsafe_code)]
use clap::{Parser, Subcommand};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "my-debug")]
#[command(about = "My Language Debugger", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive debugging session
    Run {
        /// Source file to debug
        file: PathBuf,

        /// Breakpoints (line numbers)
        #[arg(short, long, value_delimiter = ',')]
        breakpoints: Vec<usize>,
    },

    /// Attach to running process
    Attach {
        /// Process ID
        pid: u32,
    },
}

/// Debugger state
struct Debugger {
    source_file: PathBuf,
    breakpoints: HashSet<usize>,
    current_line: usize,
    variables: HashMap<String, String>,
    call_stack: Vec<String>,
    paused: bool,
}

impl Debugger {
    fn new(source_file: PathBuf, breakpoints: Vec<usize>) -> Self {
        Debugger {
            source_file,
            breakpoints: breakpoints.into_iter().collect(),
            current_line: 1,
            variables: HashMap::new(),
            call_stack: Vec::new(),
            paused: false,
        }
    }

    fn run(&mut self) -> anyhow::Result<()> {
        println!("My Language Debugger");
        println!("Debugging: {}", self.source_file.display());
        println!("Breakpoints: {:?}", self.breakpoints);
        println!();

        // Start REPL
        self.repl()
    }

    fn repl(&mut self) -> anyhow::Result<()> {
        use std::io::{self, Write};

        loop {
            print!("(my-debug) ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            match self.handle_command(input) {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn handle_command(&mut self, input: &str) -> anyhow::Result<bool> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(true);
        }

        match parts[0] {
            "help" | "h" | "?" => {
                self.print_help();
            }
            "run" | "r" => {
                println!("Starting execution...");
                self.current_line = 1;
                self.paused = false;
            }
            "continue" | "c" => {
                println!("Continuing execution...");
                self.paused = false;
            }
            "step" | "s" => {
                println!("Step: line {}", self.current_line);
                self.current_line += 1;
            }
            "next" | "n" => {
                println!("Next: line {}", self.current_line);
                self.current_line += 1;
            }
            "break" | "b" => {
                if parts.len() < 2 {
                    eprintln!("Usage: break <line_number>");
                } else if let Ok(line) = parts[1].parse::<usize>() {
                    self.breakpoints.insert(line);
                    println!("Breakpoint set at line {}", line);
                }
            }
            "delete" | "d" => {
                if parts.len() < 2 {
                    eprintln!("Usage: delete <line_number>");
                } else if let Ok(line) = parts[1].parse::<usize>() {
                    self.breakpoints.remove(&line);
                    println!("Breakpoint removed from line {}", line);
                }
            }
            "list" | "l" => {
                self.list_source();
            }
            "backtrace" | "bt" => {
                self.print_backtrace();
            }
            "print" | "p" => {
                if parts.len() < 2 {
                    eprintln!("Usage: print <variable>");
                } else {
                    self.print_variable(parts[1]);
                }
            }
            "locals" => {
                self.print_locals();
            }
            "info" | "i" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "breakpoints" | "b" => self.print_breakpoints(),
                        "locals" => self.print_locals(),
                        "stack" => self.print_backtrace(),
                        _ => eprintln!("Unknown info command: {}", parts[1]),
                    }
                } else {
                    eprintln!("Usage: info <breakpoints|locals|stack>");
                }
            }
            "quit" | "q" | "exit" => {
                println!("Exiting debugger");
                return Ok(false);
            }
            _ => {
                eprintln!("Unknown command: {}", parts[0]);
                eprintln!("Type 'help' for a list of commands");
            }
        }

        Ok(true)
    }

    fn print_help(&self) {
        println!("My Language Debugger Commands:");
        println!("  help (h, ?)          - Show this help");
        println!("  run (r)              - Start program execution");
        println!("  continue (c)         - Continue execution");
        println!("  step (s)             - Step into next line");
        println!("  next (n)             - Step over next line");
        println!("  break (b) <line>     - Set breakpoint at line");
        println!("  delete (d) <line>    - Delete breakpoint at line");
        println!("  list (l)             - List source code");
        println!("  backtrace (bt)       - Show call stack");
        println!("  print (p) <var>      - Print variable value");
        println!("  locals               - Show local variables");
        println!("  info (i) <what>      - Show information");
        println!("  quit (q, exit)       - Exit debugger");
    }

    fn list_source(&self) {
        println!("Source: {}", self.source_file.display());
        println!("Current line: {}", self.current_line);
        // In a real implementation, would read and display source file
        println!("(source listing would appear here)");
    }

    fn print_backtrace(&self) {
        println!("Call stack:");
        if self.call_stack.is_empty() {
            println!("  <empty>");
        } else {
            for (i, frame) in self.call_stack.iter().enumerate() {
                println!("  #{}: {}", i, frame);
            }
        }
    }

    fn print_variable(&self, name: &str) {
        match self.variables.get(name) {
            Some(value) => println!("{} = {}", name, value),
            None => println!("Variable not found: {}", name),
        }
    }

    fn print_locals(&self) {
        println!("Local variables:");
        if self.variables.is_empty() {
            println!("  <none>");
        } else {
            for (name, value) in &self.variables {
                println!("  {} = {}", name, value);
            }
        }
    }

    fn print_breakpoints(&self) {
        println!("Breakpoints:");
        if self.breakpoints.is_empty() {
            println!("  <none>");
        } else {
            let mut breakpoints: Vec<_> = self.breakpoints.iter().collect();
            breakpoints.sort();
            for line in breakpoints {
                println!("  Line {}", line);
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, breakpoints } => {
            let mut debugger = Debugger::new(file, breakpoints);
            debugger.run()?;
        }
        Commands::Attach { pid } => {
            println!("Attaching to process {}...", pid);
            println!("(attach not yet implemented)");
        }
    }

    Ok(())
}
