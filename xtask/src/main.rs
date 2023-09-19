use clap::{Parser, Subcommand};
use std::path::PathBuf;
use xshell::{cmd, Shell};

const APP: &str = "dot";

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run an example
    RunWeb {
        /// Directory of web application to build and serve
        #[arg(short, long)]
        dir: PathBuf,
        #[arg(short, long)]
        port: u16,
    },
}

fn build_example(sh: &Shell, dir: &PathBuf, port: u16) -> anyhow::Result<()> {
    let current_dir = sh.current_dir();
    sh.change_dir(dir);
    cmd!(sh, "wasm-pack build --target web --release").run()?;
    sh.change_dir(current_dir);
    let port = port.to_string();
    cmd!(sh, "cargo run -p serve -- --dir {dir} --port {port}").run()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let sh = Shell::new()?;
    match args.command {
        Commands::RunWeb { dir, port } => {
            build_example(&sh, &dir, port)?;
        }
    }
    Ok(())
}
