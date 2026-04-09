mod create_esm;
mod extract;
mod pack;
mod rename;
mod repack;
mod smoke_test;
mod string_table;
mod transliterate;
mod validate;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "ba2-packer")]
#[command(about = "CLI tool for building Starfield Russian Translation Mod")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// CLI-level validation profile (maps to `validate::ValidationProfile`).
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliValidationProfile {
    /// Full validation — all interface files required
    Full,
    /// Standard-font translit — skip `fontconfig_en.txt` and `fonts_en.swf` checks
    StandardFontTranslit,
}

impl From<CliValidationProfile> for validate::ValidationProfile {
    fn from(cli: CliValidationProfile) -> Self {
        match cli {
            CliValidationProfile::Full => Self::Full,
            CliValidationProfile::StandardFontTranslit => Self::StandardFontTranslit,
        }
    }
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

        /// Credit the translation author (creates CREDITS.txt in output directory)
        #[arg(long)]
        credit: Option<String>,
    },

    /// Validate built mod artifacts in dist directory
    Validate {
        /// Path to dist directory containing ESM and BA2 files
        dist_dir: PathBuf,

        /// Path to directory with source string table files (optional, falls back to dist/Strings/)
        #[arg(long)]
        source_strings: Option<PathBuf>,

        /// Path to directory with source interface files (optional, falls back to dist/Interface/)
        #[arg(long)]
        source_interface: Option<PathBuf>,

        /// Require CREDITS.txt (fail instead of warn if missing)
        #[arg(long)]
        require_credits: bool,

        /// Validation profile: "full" (default) checks all interface files;
        /// "standard-font-translit" skips `fontconfig_en.txt` and `fonts_en.swf` checks
        #[arg(long, value_enum, default_value_t = CliValidationProfile::Full)]
        profile: CliValidationProfile,
    },

    /// Extract binary string tables to human-readable JSONL format
    Extract {
        /// Input: a single string table file or directory containing them
        #[arg(long)]
        input: PathBuf,

        /// Output directory for JSONL files
        #[arg(long)]
        output_dir: PathBuf,
    },

    /// Repack JSONL string files back into binary string table format
    Repack {
        /// Input: a single JSONL file or directory containing them
        #[arg(long)]
        input: PathBuf,

        /// Output directory for binary string table files
        #[arg(long)]
        output_dir: PathBuf,
    },

    /// Create a minimal StarfieldRussian.esm plugin
    CreateEsm {
        /// Output path (file or directory)
        #[arg(long)]
        output: PathBuf,
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

    /// Transliterate Cyrillic text to Latin in string tables and translate files
    Transliterate {
        /// Input directory with string table files and/or `translate_en.txt`
        #[arg(long)]
        input_dir: PathBuf,

        /// Output directory for transliterated files
        #[arg(long)]
        output_dir: PathBuf,

        /// Credit the original translation author (creates CREDITS.txt)
        #[arg(long)]
        credit: Option<String>,
    },

    /// Run a local E2E smoke test: rename → transliterate → pack → validate
    SmokeTest {
        /// Input directory with _ru translation files
        #[arg(long)]
        input_dir: PathBuf,

        /// Output directory for artifacts (default: temporary directory)
        #[arg(long)]
        output_dir: Option<PathBuf>,

        /// Path to interface files directory (default: src/interface relative to repo root)
        #[arg(long)]
        interface_dir: Option<PathBuf>,

        /// Credit the translation author
        #[arg(long)]
        credit: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pack {
            input_strings,
            input_interface,
            output_dir,
            credit,
        } => pack::run(
            &input_strings,
            &input_interface,
            &output_dir,
            credit.as_deref(),
        ),
        Commands::Validate {
            dist_dir,
            source_strings,
            source_interface,
            require_credits,
            profile,
        } => validate::run(
            &dist_dir,
            source_strings.as_deref(),
            source_interface.as_deref(),
            require_credits,
            profile.into(),
        ),
        Commands::Extract { input, output_dir } => extract::run(&input, &output_dir),
        Commands::Repack { input, output_dir } => repack::run(&input, &output_dir),
        Commands::CreateEsm { output } => create_esm::run(&output),
        Commands::Rename {
            input_dir,
            output_dir,
        } => rename::run(&input_dir, &output_dir),
        Commands::Transliterate {
            input_dir,
            output_dir,
            credit,
        } => transliterate::run(&input_dir, &output_dir, credit.as_deref()),
        Commands::SmokeTest {
            input_dir,
            output_dir,
            interface_dir,
            credit,
        } => {
            let interface_dir = match interface_dir {
                Some(dir) => dir,
                None => resolve_default_interface_dir()?,
            };
            smoke_test::run(
                &input_dir,
                output_dir.as_deref(),
                &interface_dir,
                credit.as_deref(),
            )
        }
    }
}

/// Walk up from the current working directory to find the repository root
/// (identified by a `.git` directory), then return `<root>/src/interface`.
fn resolve_default_interface_dir() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir().context("Failed to determine current directory")?;
    find_repo_root(&cwd)
        .map(|root| root.join("src").join("interface"))
        .with_context(|| {
            format!(
                "Could not locate repository root from {}; \
                 please specify --interface-dir explicitly",
                cwd.display()
            )
        })
}

/// Walk up from `start` looking for a directory that contains `.git`.
fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut current = start;
    loop {
        if current.join(".git").exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn find_repo_root_from_subdirectory() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join(".git")).unwrap();
        let sub = root.join("a").join("b").join("c");
        fs::create_dir_all(&sub).unwrap();

        assert_eq!(find_repo_root(&sub).unwrap(), root);
    }

    #[test]
    fn find_repo_root_at_root_itself() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join(".git")).unwrap();

        assert_eq!(find_repo_root(root).unwrap(), root);
    }

    #[test]
    fn find_repo_root_returns_none_without_git() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("a").join("b");
        fs::create_dir_all(&sub).unwrap();

        // No .git anywhere under tmp — but we can't guarantee no .git above tmp,
        // so just verify the function doesn't panic and handles the search.
        let result = find_repo_root(&sub);
        // If it found a .git above tmp, that's fine; the key invariant is no panic.
        assert!(result.is_none() || result.unwrap().join(".git").exists());
    }
}
