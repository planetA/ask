use std::{fs::File, process::Output};

use anyhow::anyhow;
use clap::Parser;
use serde::{Serialize, Deserialize};
use tokio::{sync::mpsc, process::Command};

/// Indexer for askl
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Path to compile command to get the list of source files
    #[clap(value_name = "FILE")]
    compile_commands: String,

    /// Command to invoke Clang
    #[clap(short, long, default_value = "clang")]
    clang: String,

    /// Maximum parallelism
    #[clap(short, long, default_value = "1")]
    parallelism: usize,

    // Limit how many files can be processed
    #[clap(long)]
    trim: Option<usize>
}

#[derive(Serialize, Deserialize, Debug)]
struct CompileCommand {
    arguments: Option<Vec<String>>,
    command: Option<String>,
    directory: String,
    file: String,
    output: Option<String>,
}

async fn run_ast_gen(args: &Args, c: CompileCommand) -> anyhow::Result<(String, Output)> {

    let mut arguments = if let Some(ref command) = c.command {
        shell_words::split(command).expect("Failed to parse command")
    } else if let Some(arguments) = c.arguments {
        arguments
    } else {
        return Err(anyhow!("Either command or arguments must be defined for file: {}", c.file));
    };

    arguments.push("-Xclang".to_string());
    arguments.push("-ast-dump=json".to_string());
    arguments.push("-fsyntax-only".to_string());

    let output = Command::new(args.clang.clone())
        .args(&arguments[1..])
        .output().await?;

    Ok((c.file ,output))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();

    let file = File::open(&args.compile_commands)
        .expect("file should open read only");
    let mut compile_commands: Vec<CompileCommand> = serde_json::from_reader(file)
        .expect("file should be proper JSON");

    if let Some(trim) = args.trim {
        compile_commands.truncate(trim);
    }

    let (tx, mut rx) = mpsc::channel(args.parallelism);

    tokio::spawn(async move {
        for c in compile_commands {
            tx.send(c).await.unwrap();
        }
    });

    let mut first = true;
    println!("[");
    while let Some(c) = rx.recv().await {
        let (file, output) = run_ast_gen(&args, c).await?;

        if first {
            first = false;
        } else {
            println!(",");
        }
        print!(r#""{}": {}"#, file, String::from_utf8(output.stdout)?);
    }
    println!("\n]");

    Ok(())
}