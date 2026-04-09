use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{create_esm, pack, rename, transliterate, validate};

/// Run the full E2E smoke test pipeline: rename -> transliterate -> pack -> validate.
pub fn run(
    input_dir: &Path,
    output_dir: Option<&Path>,
    interface_dir: &Path,
    credit: Option<&str>,
) -> Result<()> {
    if !input_dir.is_dir() {
        bail!("Input directory does not exist: {}", input_dir.display());
    }

    if !interface_dir.is_dir() {
        bail!(
            "Interface directory does not exist: {}",
            interface_dir.display()
        );
    }

    println!("=== Smoke Test: local E2E pipeline ===\n");

    // Create temp workspace for intermediate files
    let workspace = tempfile::TempDir::new().context("Failed to create temporary workspace")?;
    let renamed_dir = workspace.path().join("renamed");
    let transliterated_dir = workspace.path().join("transliterated");
    let staged_interface_dir = workspace.path().join("staged_interface");
    fs::create_dir_all(&renamed_dir).context("Failed to create renamed dir")?;
    fs::create_dir_all(&transliterated_dir).context("Failed to create transliterated dir")?;
    fs::create_dir_all(&staged_interface_dir).context("Failed to create staged interface dir")?;

    // Determine output directory: user-specified or a persistent temp dir
    let auto_output: Option<PathBuf> = if output_dir.is_none() {
        let dir = std::env::temp_dir().join(format!("ba2-smoke-test-{}", std::process::id()));
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create output directory: {}", dir.display()))?;
        Some(dir)
    } else {
        None
    };
    let dist_dir: &Path = if let Some(dir) = output_dir {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create output directory: {}", dir.display()))?;
        dir
    } else {
        auto_output.as_deref().expect("auto_output was set above")
    };

    // Step 1: Rename _ru -> _en
    println!("Step 1/5: rename (_ru -> _en)");
    rename::run(input_dir, &renamed_dir)?;

    // Step 2: Transliterate Cyrillic -> Latin
    println!("\nStep 2/5: transliterate (Cyrillic -> Latin)");
    transliterate::run(&renamed_dir, &transliterated_dir, credit)?;

    // Stage interface: baseline from --interface-dir, then overlay with
    // pipeline-produced files so pack/validate test the actual input package.
    stage_interface(
        interface_dir,
        &renamed_dir,
        &transliterated_dir,
        &staged_interface_dir,
    )?;

    // Step 3: Create ESM
    println!("\nStep 3/5: create-esm");
    create_esm::run(dist_dir)?;

    // Step 4: Pack BA2 archives
    println!("\nStep 4/5: pack (BA2 archives)");
    pack::run(&transliterated_dir, &staged_interface_dir, dist_dir, credit)?;

    // Step 5: Validate
    println!("\nStep 5/5: validate");
    let validation_result = validate::run(
        dist_dir,
        Some(&transliterated_dir),
        Some(&staged_interface_dir),
        credit.is_some(),
        validate::ValidationProfile::Full,
    );

    // Print publish readiness summary
    println!("\n=== Publish Readiness Summary ===\n");
    print_summary(dist_dir, credit);

    println!("\nArtifacts location: {}", dist_dir.display());

    if let Err(e) = validation_result {
        bail!("Smoke test failed: {e}");
    }

    println!("\nSmoke test PASSED");
    Ok(())
}

/// Stage a combined interface directory for pack/validate.
///
/// 1. Copy baseline files from the external `interface_dir`.
/// 2. Overlay `fontconfig_en.txt` and `fonts_en.swf` from `renamed_dir` (produced by rename).
/// 3. Overlay `translate_en.txt` from `transliterated_dir` (produced by transliterate).
///
/// This ensures pack/validate operate on interface assets derived from the input
/// package rather than an unrelated external directory.
fn stage_interface(
    interface_dir: &Path,
    renamed_dir: &Path,
    transliterated_dir: &Path,
    staged_dir: &Path,
) -> Result<()> {
    // Baseline: copy all files from the external interface directory
    if let Ok(entries) = fs::read_dir(interface_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    fs::copy(&path, staged_dir.join(name)).with_context(|| {
                        format!("Failed to copy interface file: {}", path.display())
                    })?;
                }
            }
        }
    }

    // Overlay interface files produced by the rename step
    let rename_overlays = ["fontconfig_en.txt", "fonts_en.swf"];
    for name in &rename_overlays {
        let src = renamed_dir.join(name);
        if src.is_file() {
            fs::copy(&src, staged_dir.join(name)).with_context(|| {
                format!("Failed to stage renamed interface file: {}", src.display())
            })?;
        }
    }

    // Overlay translate_en.txt from the transliterate step
    let transliterated_translate = transliterated_dir.join("translate_en.txt");
    if transliterated_translate.is_file() {
        fs::copy(
            &transliterated_translate,
            staged_dir.join("translate_en.txt"),
        )
        .with_context(|| {
            format!(
                "Failed to stage transliterated interface file: {}",
                transliterated_translate.display()
            )
        })?;
    }

    Ok(())
}

fn print_summary(dist_dir: &Path, credit: Option<&str>) {
    let artifacts = [
        "StarfieldRussian.esm",
        "StarfieldRussian - Main.ba2",
        "StarfieldRussian - Interface.ba2",
    ];

    let mut total_size: u64 = 0;
    for name in &artifacts {
        let path = dist_dir.join(name);
        if path.exists() {
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            total_size += size;
            println!("  [OK] {name} ({size} bytes)");
        } else {
            println!("  [MISSING] {name}");
        }
    }
    println!("  Total size: {total_size} bytes");

    // Credit presence
    let credits_path = dist_dir.join("CREDITS.txt");
    if credits_path.exists() {
        println!("  Credits: present ({})", credit.unwrap_or("unknown"));
    } else if credit.is_some() {
        println!("  Credits: MISSING (expected due to --credit flag)");
    } else {
        println!("  Credits: not required");
    }

    // PS5 font warning
    println!(
        "\n  WARNING: PS5 font pre-loading via sResourceStartUpArchiveList \
         cannot be configured through Creations."
    );
    println!("  If Cyrillic fonts are not pre-loaded, text will render as boxes.");
}
