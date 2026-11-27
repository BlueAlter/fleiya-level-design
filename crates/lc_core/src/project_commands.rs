//! Undoable commands for modifying project metadata and global project settings.

use crate::command::{Command, DocKind};
use crate::error::CoreError;
use crate::project::{Project, ProjectSettings};
use std::any::Any;

/// Renames the active project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameProjectCommand {
    /// Target project name (trimmed on apply).
    pub new_name: String,
    old_name: Option<String>,
}

impl RenameProjectCommand {
    /// Creates a new [`RenameProjectCommand`].
    pub fn new(new_name: impl Into<String>) -> Self {
        Self {
            new_name: new_name.into(),
            old_name: None,
        }
    }
}

impl Command for RenameProjectCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let trimmed = self.new_name.trim();
        if trimmed.is_empty() {
            return Err(CoreError::DefNotFound(
                "Project name cannot be empty".to_string(),
            ));
        }
        if self.old_name.is_none() {
            self.old_name = Some(p.name.clone());
        }
        p.name = trimmed.to_string();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(old) = &self.old_name {
            p.name = old.clone();
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(next_cmd) = next.as_any().downcast_ref::<Self>() {
            self.new_name = next_cmd.new_name.clone();
            return true;
        }
        false
    }

    fn label(&self) -> String {
        "Rename Project".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::ProjectSettings)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Updates [`ProjectSettings`] for the active project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateProjectSettingsCommand {
    /// Replacement settings values.
    pub new_settings: ProjectSettings,
    old_settings: Option<ProjectSettings>,
}

impl UpdateProjectSettingsCommand {
    /// Creates a new [`UpdateProjectSettingsCommand`].
    pub const fn new(new_settings: ProjectSettings) -> Self {
        Self {
            new_settings,
            old_settings: None,
        }
    }
}

impl Command for UpdateProjectSettingsCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        if self.new_settings.default_grid_size.x == 0 || self.new_settings.default_grid_size.y == 0
        {
            return Err(CoreError::InvalidGridSize {
                expected: 1,
                actual: 0,
            });
        }
        if self.old_settings.is_none() {
            self.old_settings = Some(p.settings.clone());
        }
        p.settings = self.new_settings.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(old) = &self.old_settings {
            p.settings = old.clone();
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(next_cmd) = next.as_any().downcast_ref::<Self>() {
            self.new_settings = next_cmd.new_settings.clone();
            return true;
        }
        false
    }

    fn label(&self) -> String {
        "Update Project Settings".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::ProjectSettings)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::UVec2;

    #[test]
    fn rename_project_apply_revert_and_merge() {
        let mut project = Project::new("Original");
        let baseline = project.clone();

        let mut first = RenameProjectCommand::new("Renamed");
        assert!(first.apply(&mut project).is_ok());
        assert_eq!(project.name, "Renamed");

        let second = RenameProjectCommand::new("  Final Name  ");
        assert!(first.merge(&second));
        assert!(first.apply(&mut project).is_ok());
        assert_eq!(project.name, "Final Name");

        // Verify empty name fails cleanly without mutating project.
        let mut invalid = RenameProjectCommand::new("   ");
        assert!(invalid.apply(&mut project).is_err());
        assert_eq!(project.name, "Final Name");

        first.revert(&mut project);
        assert_eq!(project, baseline);
    }

    #[test]
    fn update_project_settings_apply_revert_and_merge() {
        let mut project = Project::new("SettingsTest");
        let baseline = project.clone();

        let mut first = UpdateProjectSettingsCommand::new(ProjectSettings {
            default_grid_size: UVec2::new(24, 24),
        });
        assert!(first.apply(&mut project).is_ok());
        assert_eq!(project.settings.default_grid_size, UVec2::new(24, 24));

        let next = UpdateProjectSettingsCommand::new(ProjectSettings {
            default_grid_size: UVec2::new(32, 16),
        });
        assert!(first.merge(&next));
        assert!(first.apply(&mut project).is_ok());
        assert_eq!(project.settings.default_grid_size, UVec2::new(32, 16));

        let mut invalid = UpdateProjectSettingsCommand::new(ProjectSettings {
            default_grid_size: UVec2::new(0, 16),
        });
        assert!(invalid.apply(&mut project).is_err());

        first.revert(&mut project);
        assert_eq!(project, baseline);
    }
}
