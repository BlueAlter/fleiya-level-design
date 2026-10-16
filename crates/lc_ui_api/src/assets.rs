//! Asset and texture management services for feature crates.
//!
//! Provides texture loading without requiring direct disk access or `lc_io`.

/// Typed errors that can occur during asset resolution or decoding.
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    /// The requested asset file was not found.
    #[error("Asset not found at path: {0}")]
    NotFound(String),
    /// An I/O error occurred while reading asset bytes.
    #[error("I/O error loading asset: {0}")]
    Io(String),
    /// Failed to decode image data into pixels.
    #[error("Failed to decode image data: {0}")]
    Decode(String),
    /// The specified asset path was malformed or outside the project root.
    #[error("Invalid asset path: {0}")]
    InvalidPath(String),
}

/// Service interface for loading and caching image textures in the editor UI.
pub trait AssetService: Send + Sync {
    /// Loads a texture from a project-relative or absolute file path.
    ///
    /// Results are cached by path. Successive calls return the cached handle.
    fn load_texture(
        &self,
        ctx: &egui::Context,
        relative_path: &str,
    ) -> Result<egui::TextureHandle, AssetError>;

    /// Invalidates and evicts the cached texture for the given path.
    fn invalidate(&self, relative_path: &str);
}

/// Fallback no-op asset service for headless environments or unit tests.
#[derive(Debug, Default)]
pub struct NoopAssetService;

impl AssetService for NoopAssetService {
    fn load_texture(
        &self,
        _ctx: &egui::Context,
        relative_path: &str,
    ) -> Result<egui::TextureHandle, AssetError> {
        Err(AssetError::NotFound(relative_path.to_string()))
    }

    fn invalidate(&self, _relative_path: &str) {}
}
