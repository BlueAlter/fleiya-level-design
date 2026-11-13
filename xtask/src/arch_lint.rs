use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDepInfo {
    pub name: String,
    pub dependencies: Vec<String>,
}

/// Validates that workspace crates adhere to the dependency matrix and third-party restrictions
/// specified in `docs/02-architecture.md`.
pub fn validate_dependencies(packages: &[PackageDepInfo]) -> Vec<String> {
    let mut errors = Vec::new();

    for pkg in packages {
        let name = pkg.name.as_str();

        // Skip non-runtime packages like xtask
        if name == "xtask" {
            continue;
        }

        let deps: HashSet<&str> = pkg.dependencies.iter().map(|s| s.as_str()).collect();

        // 1. Workspace internal crate dependencies check
        match name {
            "lc_core" => {
                for dep in &deps {
                    if dep.starts_with("lc_") {
                        errors.push(format!(
                            "Architecture violation in `{}`: cannot depend on workspace crate `{}`",
                            name, dep
                        ));
                    }
                }
            }
            "lc_io" | "lc_ui_api" => {
                for dep in &deps {
                    if dep.starts_with("lc_") && *dep != "lc_core" {
                        errors.push(format!(
                            "Architecture violation in `{}`: may only depend on `lc_core`, found `{}`",
                            name, dep
                        ));
                    }
                }
            }
            "lc_canvas" => {
                for dep in &deps {
                    if dep.starts_with("lc_") && *dep != "lc_core" && *dep != "lc_ui_api" {
                        errors.push(format!(
                            "Architecture violation in `{}`: may only depend on `lc_core` and `lc_ui_api`, found `{}`",
                            name, dep
                        ));
                    }
                }
            }
            _ if name.starts_with("lc_feature_") => {
                for dep in &deps {
                    if dep.starts_with("lc_feature_") && *dep != name {
                        errors.push(format!(
                            "Architecture violation in `{}`: cross-feature dependency on `{}` is strictly forbidden",
                            name, dep
                        ));
                    } else if *dep == "lc_io" || *dep == "lc_shell" {
                        errors.push(format!(
                            "Architecture violation in `{}`: feature crates cannot depend on `{}`",
                            name, dep
                        ));
                    } else if dep.starts_with("lc_")
                        && *dep != "lc_core"
                        && *dep != "lc_ui_api"
                        && *dep != "lc_canvas"
                    {
                        errors.push(format!(
                            "Architecture violation in `{}`: may only depend on core, ui_api, canvas; found `{}`",
                            name, dep
                        ));
                    }
                }
            }
            "lc_shell" => {
                if deps.contains("lc_app") {
                    errors.push(
                        "Architecture violation in `lc_shell`: cannot depend on `lc_app`"
                            .to_string(),
                    );
                }
            }
            "lc_app" => {
                for dep in &deps {
                    if dep.starts_with("lc_") && *dep != "lc_shell" {
                        errors.push(format!(
                            "Architecture violation in `lc_app`: may only depend on `lc_shell`, found `{}`",
                            dep
                        ));
                    }
                }
            }
            _ => {}
        }

        // 2. Third-party dependency rules
        // egui / eframe allowed only in ui_api, canvas, features, shell, app.
        if deps.contains("egui") && (name == "lc_core" || name == "lc_io") {
            errors.push(format!(
                "Third-party violation in `{}`: `egui` is forbidden in `lc_core` and `lc_io`",
                name
            ));
        }

        if deps.contains("eframe") && name != "lc_app" {
            errors.push(format!(
                "Third-party violation in `{}`: `eframe` is only allowed in `lc_app`",
                name
            ));
        }

        // egui_dock only in shell
        if deps.contains("egui_dock") && name != "lc_shell" {
            errors.push(format!(
                "Third-party violation in `{}`: `egui_dock` is only allowed in `lc_shell`",
                name
            ));
        }

        // egui_commonmark only in shell (ADR-0016)
        if deps.contains("egui_commonmark") && name != "lc_shell" {
            errors.push(format!(
                "Third-party violation in `{}`: `egui_commonmark` is only allowed in `lc_shell`",
                name
            ));
        }

        // serde_json only in io
        if deps.contains("serde_json") && name != "lc_io" {
            errors.push(format!(
                "Third-party violation in `{}`: `serde_json` is only allowed in `lc_io`",
                name
            ));
        }
    }

    errors
}

/// Validates that `std::fs` is NOT used anywhere outside of `lc_io` in `crates/`.
pub fn validate_fs_usage(crates_dir: &Path) -> Result<Vec<String>> {
    let mut errors = Vec::new();

    if !crates_dir.exists() {
        return Ok(errors);
    }

    for entry in walkdir(crates_dir)? {
        let path = entry.as_path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        // Allow lc_io
        if path.components().any(|c| c.as_os_str() == "lc_io") {
            continue;
        }

        let content = std::fs::read_to_string(path)?;
        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Ignore comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            if line.contains("std::fs") || line.contains("use std::fs") {
                errors.push(format!(
                    "Forbidden `std::fs` usage in {}:{}: {}",
                    path.display(),
                    line_no + 1,
                    trimmed
                ));
            }
        }
    }

    Ok(errors)
}

fn walkdir(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut results = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                results.extend(walkdir(&path)?);
            } else {
                results.push(path);
            }
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_workspace_passes() {
        let packages = vec![
            PackageDepInfo {
                name: "lc_core".to_string(),
                dependencies: vec!["thiserror".to_string(), "glam".to_string()],
            },
            PackageDepInfo {
                name: "lc_io".to_string(),
                dependencies: vec![
                    "lc_core".to_string(),
                    "serde_json".to_string(),
                    "thiserror".to_string(),
                ],
            },
            PackageDepInfo {
                name: "lc_ui_api".to_string(),
                dependencies: vec![
                    "lc_core".to_string(),
                    "egui".to_string(),
                    "thiserror".to_string(),
                ],
            },
            PackageDepInfo {
                name: "lc_canvas".to_string(),
                dependencies: vec![
                    "lc_core".to_string(),
                    "lc_ui_api".to_string(),
                    "egui".to_string(),
                ],
            },
            PackageDepInfo {
                name: "lc_feature_level".to_string(),
                dependencies: vec![
                    "lc_core".to_string(),
                    "lc_ui_api".to_string(),
                    "lc_canvas".to_string(),
                    "egui".to_string(),
                ],
            },
            PackageDepInfo {
                name: "lc_shell".to_string(),
                dependencies: vec![
                    "lc_core".to_string(),
                    "lc_io".to_string(),
                    "lc_ui_api".to_string(),
                    "lc_canvas".to_string(),
                    "lc_feature_level".to_string(),
                    "egui".to_string(),
                    "egui_dock".to_string(),
                    "egui_commonmark".to_string(),
                ],
            },
            PackageDepInfo {
                name: "lc_app".to_string(),
                dependencies: vec![
                    "lc_shell".to_string(),
                    "eframe".to_string(),
                    "egui".to_string(),
                ],
            },
        ];

        let errors = validate_dependencies(&packages);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_detect_forbidden_core_dependency() {
        let packages = vec![PackageDepInfo {
            name: "lc_core".to_string(),
            dependencies: vec!["lc_io".to_string()],
        }];
        let errors = validate_dependencies(&packages);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("cannot depend on workspace crate `lc_io`"));
    }

    #[test]
    fn test_detect_cross_feature_dependency() {
        let packages = vec![PackageDepInfo {
            name: "lc_feature_level".to_string(),
            dependencies: vec!["lc_feature_tileset".to_string()],
        }];
        let errors = validate_dependencies(&packages);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("cross-feature dependency on `lc_feature_tileset`"));
    }

    #[test]
    fn test_detect_feature_depending_on_io() {
        let packages = vec![PackageDepInfo {
            name: "lc_feature_level".to_string(),
            dependencies: vec!["lc_io".to_string()],
        }];
        let errors = validate_dependencies(&packages);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("feature crates cannot depend on `lc_io`"));
    }

    #[test]
    fn test_detect_egui_in_core() {
        let packages = vec![PackageDepInfo {
            name: "lc_core".to_string(),
            dependencies: vec!["egui".to_string()],
        }];
        let errors = validate_dependencies(&packages);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("`egui` is forbidden in `lc_core`"));
    }

    #[test]
    fn test_detect_egui_dock_outside_shell() {
        let packages = vec![PackageDepInfo {
            name: "lc_feature_tileset".to_string(),
            dependencies: vec!["egui_dock".to_string()],
        }];
        let errors = validate_dependencies(&packages);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("`egui_dock` is only allowed in `lc_shell`"));
    }

    #[test]
    fn test_detect_egui_commonmark_outside_shell() {
        let packages = vec![PackageDepInfo {
            name: "lc_feature_enums".to_string(),
            dependencies: vec!["egui_commonmark".to_string()],
        }];
        let errors = validate_dependencies(&packages);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("`egui_commonmark` is only allowed in `lc_shell`"));
    }

    #[test]
    fn test_detect_serde_json_outside_io() {
        let packages = vec![PackageDepInfo {
            name: "lc_core".to_string(),
            dependencies: vec!["serde_json".to_string()],
        }];
        let errors = validate_dependencies(&packages);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("`serde_json` is only allowed in `lc_io`"));
    }

    #[test]
    fn test_detect_eframe_outside_app() {
        let packages = vec![PackageDepInfo {
            name: "lc_shell".to_string(),
            dependencies: vec!["eframe".to_string()],
        }];
        let errors = validate_dependencies(&packages);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("`eframe` is only allowed in `lc_app`"));
    }

    #[test]
    fn test_detect_fs_usage_violation() {
        let temp_dir = std::env::temp_dir().join("xtask_test_fs_violation");
        let core_src = temp_dir.join("lc_core/src");
        let io_src = temp_dir.join("lc_io/src");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&core_src).unwrap();
        std::fs::create_dir_all(&io_src).unwrap();

        // Allowed in io
        std::fs::write(io_src.join("lib.rs"), "use std::fs::File;\n").unwrap();
        // Forbidden in core
        std::fs::write(
            core_src.join("lib.rs"),
            "use std::fs::read;\nfn foo() { std::fs::write(\"a\", \"b\"); }\n",
        )
        .unwrap();

        let errors = validate_fs_usage(&temp_dir).unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);

        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("Forbidden `std::fs`"));
        assert!(errors[1].contains("Forbidden `std::fs`"));
    }
}
