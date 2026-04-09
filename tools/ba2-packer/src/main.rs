mod pack;
mod rename;
mod validate;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ba2-packer")]
#[command(about = "CLI tool for building Starfield Russian Translation Mod")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pack source files into BA2 archives
    Pack {
        /// Path to directory with string table files
        #[arg(long)]
        input_strings: PathBuf,

        /// Path to directory with interface files (fonts, fontconfig, translate)
        #[arg(long)]
        input_interface: PathBuf,

        /// Output directory for BA2 archives
        #[arg(long)]
        output_dir: PathBuf,
    },

    /// Validate built mod artifacts in dist directory
    Validate {
        /// Path to dist directory containing ESM and BA2 files
        dist_dir: PathBuf,
    },

    /// Rename files from _ru to _en naming convention
    Rename {
        /// Input directory with _ru named files
        #[arg(long)]
        input_dir: PathBuf,

        /// Output directory for _en named files
        #[arg(long)]
        output_dir: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pack {
            input_strings,
            input_interface,
            output_dir,
        } => pack::run(&input_strings, &input_interface, &output_dir),
        Commands::Validate { dist_dir } => validate::run(&dist_dir),
        Commands::Rename {
            input_dir,
            output_dir,
        } => rename::run(&input_dir, &output_dir),
    }
}
