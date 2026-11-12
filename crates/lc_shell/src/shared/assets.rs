//! Asset service implementation for loading and caching image textures in the editor shell.

use egui::ColorImage;
use lc_ui_api::assets::{AssetError, AssetService};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

/// Shell asset manager resolving paths against the project root and caching loaded GPU textures.
#[derive(Clone)]
pub struct ShellAssetService {
    project_dir: Arc<RwLock<Option<PathBuf>>>,
    cache: Arc<Mutex<HashMap<String, egui::TextureHandle>>>,
}

impl std::fmt::Debug for ShellAssetService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.cache.lock().map(|c| c.len()).unwrap_or(0);
        f.debug_struct("ShellAssetService")
            .field("project_dir", &self.project_dir)
            .field("cached_textures_count", &count)
            .finish()
    }
}

impl Default for ShellAssetService {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellAssetService {
    /// Creates a new asset service with an empty cache.
    pub fn new() -> Self {
        Self {
            project_dir: Arc::new(RwLock::new(None)),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Sets or updates the active project base directory for resolving relative paths.
    pub fn set_project_dir(&self, dir: Option<PathBuf>) {
        if let Ok(mut lock) = self.project_dir.write() {
            *lock = dir;
        }
    }

    /// Resolves a relative path to a full path on disk.
    pub fn resolve_path(&self, relative_path: &str) -> PathBuf {
        let path = Path::new(relative_path);
        if path.is_absolute() {
            path.to_path_buf()
        } else if let Ok(lock) = self.project_dir.read() {
            if let Some(ref base) = *lock {
                base.join(path)
            } else {
                path.to_path_buf()
            }
        } else {
            path.to_path_buf()
        }
    }
}

impl AssetService for ShellAssetService {
    fn load_texture(
        &self,
        ctx: &egui::Context,
        relative_path: &str,
    ) -> Result<egui::TextureHandle, AssetError> {
        if relative_path.trim().is_empty() {
            return Err(AssetError::InvalidPath(
                "Asset path cannot be empty".to_string(),
            ));
        }

        // 1. Check in-memory cache
        if let Ok(cache) = self.cache.lock() {
            if let Some(handle) = cache.get(relative_path) {
                return Ok(handle.clone());
            }
        }

        // 2. Read bytes using lc_io (never std::fs directly in shell)
        let full_path = self.resolve_path(relative_path);
        let bytes = lc_io::read_bytes(&full_path).map_err(|e| match e {
            lc_io::IoError::Fs { path, source } => {
                if source.kind() == std::io::ErrorKind::NotFound {
                    AssetError::NotFound(path.display().to_string())
                } else {
                    AssetError::Io(format!("{}: {source}", path.display()))
                }
            }
            other => AssetError::Io(other.to_string()),
        })?;

        // 3. Decode image from bytes
        let img = image::load_from_memory(&bytes)
            .map_err(|e| AssetError::Decode(format!("{}: {e}", full_path.display())))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let size = [width as usize, height as usize];
        let pixels = rgba.into_raw();

        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

        // 4. Allocate egui texture handle
        let handle = ctx.load_texture(
            relative_path,
            color_image,
            egui::TextureOptions::NEAREST, // Pixel art default
        );

        // 5. Cache handle
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(relative_path.to_string(), handle.clone());
        }

        Ok(handle)
    }

    fn invalidate(&self, relative_path: &str) {
        if let Ok(mut cache) = self.cache.lock() {
            if relative_path == "*" || relative_path.is_empty() {
                cache.clear();
            } else {
                cache.remove(relative_path);
            }
        }
    }
}
