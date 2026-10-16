mod arch_lint;

use anyhow::{bail, Context, Result};
use arch_lint::{validate_dependencies, validate_fs_usage, PackageDepInfo};
use std::process::Command;
use tracing::{error, info, warn};

fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    info!("-> Running `{} {}`...", cmd, args.join(" "));
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("Failed to spawn command `{}`", cmd))?;

    if !status.success() {
        bail!(
            "Command `{} {}` failed with exit status: {}",
            cmd,
            args.join(" "),
            status
        );
    }
    Ok(())
}

fn check() -> Result<()> {
    info!("=== Gate 1: cargo fmt --all -- --check ===");
    run_command("cargo", &["fmt", "--all", "--", "--check"])
        .context("Gate failed: cargo fmt check failed. Run `cargo fmt --all` to fix.")?;

    info!("=== Gate 2: cargo clippy --workspace --all-targets -- -D warnings ===");
    run_command(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )
    .context("Gate failed: clippy reported warnings or errors.")?;

    info!("=== Gate 3: cargo test --workspace ===");
    run_command("cargo", &["test", "--workspace"])
        .context("Gate failed: workspace tests failed.")?;

    info!("=== Gate 4: Architecture & Crate Boundaries Lint ===");
    let metadata = cargo_metadata::MetadataCommand::new()
        .exec()
        .context("Failed to run `cargo metadata`")?;

    let workspace_packages: Vec<PackageDepInfo> = metadata
        .workspace_packages()
        .into_iter()
        .map(|pkg| PackageDepInfo {
            name: pkg.name.to_string(),
            dependencies: pkg.dependencies.iter().map(|d| d.name.clone()).collect(),
        })
        .collect();

    let mut lint_errors = validate_dependencies(&workspace_packages);

    let crates_dir = metadata.workspace_root.as_std_path().join("crates");
    let fs_errors = validate_fs_usage(&crates_dir)
        .context("Failed while scanning crates directory for std::fs usage")?;
    lint_errors.extend(fs_errors);

    if !lint_errors.is_empty() {
        error!("Architecture lint violations found:");
        for err in &lint_errors {
            error!("  - {}", err);
        }
        bail!(
            "Gate failed: {} architecture lint violation(s) found.",
            lint_errors.len()
        );
    }

    info!("=== ALL GATES PASSED: fmt, clippy, test, architecture lint ===");
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .init();

    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("check");

    match subcommand {
        "check" => check()?,
        "board" => {
            run_command("./scripts/board.sh", &[])?;
        }
        other => {
            warn!("Unknown xtask subcommand: `{}`", other);
            info!("Available subcommands: check, board");
            std::process::exit(1);
        }
    }

    Ok(())
}
