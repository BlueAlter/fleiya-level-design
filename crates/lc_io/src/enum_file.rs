//! File format, serialization, deserialization, validation, and migration for `.fleiyaenum` files.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use lc_core::defs::{EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::layer::TileRef;
use serde::{Deserialize, Serialize};

use crate::error::IoError;

pub const ENUM_FILE_FORMAT: &str = "fleiya.enum";
pub const CURRENT_ENUM_FILE_VERSION: u32 = 1;
pub const ENUM_FILE_EXTENSION: &str = "fleiyaenum";

/// Root container for a `.fleiyaenum` file conforming to LDtk structure.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FleiyaEnumFile {
    pub format: String,
    pub version: u32,
    pub enums: Vec<FleiyaEnumDef>,
}

/// An enum definition in the `.fleiyaenum` file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FleiyaEnumDef {
    pub identifier: String,
    pub uid: i64,
    #[serde(default, rename = "iconTilesetUid")]
    pub icon_tileset_uid: Option<i64>,
    pub values: Vec<FleiyaEnumValueDef>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, rename = "externalRelPath")]
    pub external_rel_path: Option<String>,
    #[serde(default, rename = "externalFileChecksum")]
    pub external_file_checksum: Option<String>,
}

/// A value definition within an enum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FleiyaEnumValueDef {
    pub id: String,
    pub color: u32,
    #[serde(default, rename = "tileRect")]
    pub tile_rect: Option<FleiyaTileRect>,
}

/// Bound tile pixel rectangle reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleiyaTileRect {
    #[serde(rename = "tilesetUid")]
    pub tileset_uid: i64,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Strategy for resolving identifier conflicts during import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictResolutionChoice {
    Replace,
    KeepBoth,
    Skip,
}

/// Information about an identifier conflict between an imported enum and project enum.
#[derive(Clone, Debug)]
pub struct EnumConflict {
    pub file_enum: FleiyaEnumDef,
    pub existing_id: EnumDefId,
    pub existing_name: String,
    pub existing_values: Vec<EnumValueDef>,
    pub entity_count: usize,
    pub field_count: usize,
    pub choice: ConflictResolutionChoice,
}

/// Converts RGBA bytes to integer 0xRRGGBB.
pub fn rgba_to_int(rgba: [u8; 4]) -> u32 {
    let r = rgba.first().copied().unwrap_or(0) as u32;
    let g = rgba.get(1).copied().unwrap_or(0) as u32;
    let b = rgba.get(2).copied().unwrap_or(0) as u32;
    (r << 16) | (g << 8) | b
}

/// Converts integer 0xRRGGBB to RGBA bytes [r, g, b, 255].
pub fn int_to_rgba(val: u32) -> [u8; 4] {
    let r = ((val >> 16) & 0xFF) as u8;
    let g = ((val >> 8) & 0xFF) as u8;
    let b = (val & 0xFF) as u8;
    [r, g, b, 255]
}

/// Formats integer 0xRRGGBB as "#rrggbb" hex string.
pub fn int_to_hex(val: u32) -> String {
    format!("#{:06x}", val & 0xFFFFFF)
}

/// Parses hex string ("#rrggbb" or "rrggbb") to integer 0xRRGGBB.
pub fn hex_to_int(hex: &str) -> Option<u32> {
    let s = hex.strip_prefix('#').unwrap_or(hex);
    if s.len() == 6 {
        u32::from_str_radix(s, 16).ok()
    } else {
        None
    }
}

/// Formats RGBA bytes as a `#rrggbb` hex string.
pub fn rgba_to_hex(color: [u8; 4]) -> String {
    let r = color.first().copied().unwrap_or(0);
    let g = color.get(1).copied().unwrap_or(0);
    let b = color.get(2).copied().unwrap_or(0);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Parses a hex color string (`#rrggbb` or `#rrggbbaa`) into RGBA bytes.
pub fn parse_hex_color(hex: &str) -> Option<[u8; 4]> {
    let s = hex.strip_prefix('#').unwrap_or(hex);
    match s.len() {
        6 => {
            let r = u8::from_str_radix(s.get(0..2)?, 16).ok()?;
            let g = u8::from_str_radix(s.get(2..4)?, 16).ok()?;
            let b = u8::from_str_radix(s.get(4..6)?, 16).ok()?;
            Some([r, g, b, 255])
        }
        8 => {
            let r = u8::from_str_radix(s.get(0..2)?, 16).ok()?;
            let g = u8::from_str_radix(s.get(2..4)?, 16).ok()?;
            let b = u8::from_str_radix(s.get(4..6)?, 16).ok()?;
            let a = u8::from_str_radix(s.get(6..8)?, 16).ok()?;
            Some([r, g, b, a])
        }
        _ => None,
    }
}

/// Reads image dimensions (width, height) directly from the PNG header without full decoding.
pub fn read_png_dimensions(path: &Path) -> Option<(u32, u32)> {
    let mut file = fs::File::open(path).ok()?;
    use std::io::Read;
    let mut buf = [0u8; 24];
    file.read_exact(&mut buf).ok()?;
    if buf.get(0..8) == Some(b"\x89PNG\r\n\x1a\n") && buf.get(12..16) == Some(b"IHDR") {
        let w_bytes: [u8; 4] = buf.get(16..20)?.try_into().ok()?;
        let h_bytes: [u8; 4] = buf.get(20..24)?.try_into().ok()?;
        Some((u32::from_be_bytes(w_bytes), u32::from_be_bytes(h_bytes)))
    } else {
        None
    }
}

/// Converts a `tile_id` into a `FleiyaTileRect` pixel rectangle using the tileset's grid parameters.
pub fn tile_id_to_rect(
    tileset: &TilesetDef,
    tile_id: u32,
    image_size: (u32, u32),
) -> FleiyaTileRect {
    let tile_w = tileset.tile_size.x.max(1);
    let tile_h = tileset.tile_size.y.max(1);
    let padding = tileset.padding;
    let spacing = tileset.spacing;
    let (img_w, _img_h) = image_size;
    let eff_w = img_w.saturating_sub(padding * 2);
    let stride_x = tile_w + spacing;
    let stride_y = tile_h + spacing;
    let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(1).max(1);

    let col = tile_id % cols;
    let row = tile_id / cols;

    FleiyaTileRect {
        tileset_uid: tileset.id.0 as i64,
        x: padding + col * stride_x,
        y: padding + row * stride_y,
        w: tile_w,
        h: tile_h,
    }
}

/// Converts a `FleiyaTileRect` pixel rectangle back to a `tile_id`, verifying bounds and grid alignment.
pub fn rect_to_tile_id(
    tileset: &TilesetDef,
    rect: &FleiyaTileRect,
    image_size: (u32, u32),
) -> Option<u32> {
    if rect.tileset_uid != tileset.id.0 as i64 {
        return None;
    }

    let tile_w = tileset.tile_size.x.max(1);
    let tile_h = tileset.tile_size.y.max(1);
    let padding = tileset.padding;
    let spacing = tileset.spacing;
    let (img_w, img_h) = image_size;

    if rect.w == 0 || rect.h == 0 {
        return None;
    }

    if rect.x.checked_add(rect.w)? > img_w || rect.y.checked_add(rect.h)? > img_h {
        return None;
    }

    if rect.w != tile_w || rect.h != tile_h {
        return None;
    }

    let stride_x = tile_w + spacing;
    let stride_y = tile_h + spacing;
    if rect.x < padding || !(rect.x - padding).is_multiple_of(stride_x) {
        return None;
    }
    if rect.y < padding || !(rect.y - padding).is_multiple_of(stride_y) {
        return None;
    }

    let col = (rect.x - padding) / stride_x;
    let row = (rect.y - padding) / stride_y;

    let eff_w = img_w.saturating_sub(padding * 2);
    let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(1).max(1);

    Some(row * cols + col)
}

/// Validates file content and extension, collecting all validation errors in one pass.
pub fn validate_enum_file_content(
    content: &str,
    file_name: Option<&str>,
) -> Result<FleiyaEnumFile, Vec<String>> {
    let mut errors = Vec::new();

    if let Some(name) = file_name {
        let is_valid_ext = Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| ext.eq_ignore_ascii_case(ENUM_FILE_EXTENSION))
            .unwrap_or(false);

        if !is_valid_ext {
            errors.push(format!(
                "Unsupported extension: '{}'. Only .fleiyaenum is accepted.",
                name
            ));
        }
    }

    if content.trim().is_empty() {
        errors.push("File is empty.".to_string());
        return Err(errors);
    }

    let parsed_val: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!(
                "Broken JSON at line {}, column {}: {}",
                e.line(),
                e.column(),
                e
            ));
            return Err(errors);
        }
    };

    let obj = match parsed_val.as_object() {
        Some(o) => o,
        None => {
            errors.push("Root JSON must be an object.".to_string());
            return Err(errors);
        }
    };

    match obj.get("format").and_then(|v| v.as_str()) {
        Some(fmt) if fmt == ENUM_FILE_FORMAT => {}
        _ => {
            errors.push(format!("Field 'format' is not '{ENUM_FILE_FORMAT}'."));
        }
    }

    match obj.get("version").and_then(|v| v.as_u64()) {
        Some(ver) if ver as u32 > CURRENT_ENUM_FILE_VERSION => {
            errors.push(format!(
                "File version ({}) is newer than supported ({}).",
                ver, CURRENT_ENUM_FILE_VERSION
            ));
        }
        Some(_) => {}
        None => {
            errors.push("Missing or invalid 'version' field.".to_string());
        }
    }

    let enums_arr = match obj.get("enums").and_then(|v| v.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        Some(_) => {
            errors.push("The 'enums' list is empty.".to_string());
            return Err(errors);
        }
        None => {
            errors.push("The 'enums' list is empty.".to_string());
            return Err(errors);
        }
    };

    let mut seen_identifiers: HashSet<String> = HashSet::new();

    for (enum_idx, enum_val) in enums_arr.iter().enumerate() {
        let n = enum_idx + 1;
        let enum_obj = match enum_val.as_object() {
            Some(o) => o,
            None => {
                errors.push(format!("Enum #{n} must be an object."));
                continue;
            }
        };

        let identifier = enum_obj
            .get("identifier")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        if identifier.is_empty() {
            errors.push(format!("Enum #{n}: field 'identifier' is empty."));
        } else if !seen_identifiers.insert(identifier.to_string()) {
            let prev_n = enums_arr
                .iter()
                .take(enum_idx)
                .position(|e| {
                    e.get("identifier")
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim())
                        == Some(identifier)
                })
                .map(|p| p + 1)
                .unwrap_or(1);
            errors.push(format!(
                "Enum #{n}: identifier '{identifier}' duplicates Enum #{prev_n}."
            ));
        }

        let values_arr = match enum_obj.get("values").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => {
                errors.push(format!(
                    "Enum #{n} '{}': missing 'values' array.",
                    identifier
                ));
                continue;
            }
        };

        let mut seen_keys: HashSet<String> = HashSet::new();

        for (val_idx, val_item) in values_arr.iter().enumerate() {
            let m = val_idx + 1;
            let val_obj = match val_item.as_object() {
                Some(o) => o,
                None => {
                    errors.push(format!(
                        "Enum #{n} '{identifier}' > value #{m}: must be an object."
                    ));
                    continue;
                }
            };

            let val_id = val_obj
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();

            if val_id.is_empty() {
                errors.push(format!(
                    "Enum #{n} '{identifier}' > value #{m}: field 'id' is empty."
                ));
            } else if !seen_keys.insert(val_id.to_string()) {
                let prev_m = values_arr
                    .iter()
                    .take(val_idx)
                    .position(|v| {
                        v.get("id").and_then(|k| k.as_str()).map(|s| s.trim()) == Some(val_id)
                    })
                    .map(|p| p + 1)
                    .unwrap_or(1);
                errors.push(format!(
                    "Enum #{n} '{identifier}' > value #{m} '{val_id}': duplicate value id (values #{prev_m} and #{m})."
                ));
            }

            // Check color integer 0..=0xFFFFFF
            match val_obj.get("color") {
                Some(c_val) => {
                    if let Some(c_num) = c_val.as_i64() {
                        if !(0..=0xFFFFFF).contains(&c_num) {
                            errors.push(format!(
                                "Enum #{n} '{identifier}' > value #{m} '{val_id}': color {c_num} is outside 0..=0xFFFFFF."
                            ));
                        }
                    } else if let Some(c_str) = c_val.as_str() {
                        if let Some(c_num) = hex_to_int(c_str) {
                            if c_num > 0xFFFFFF {
                                errors.push(format!(
                                    "Enum #{n} '{identifier}' > value #{m} '{val_id}': color {c_num} is outside 0..=0xFFFFFF."
                                ));
                            }
                        } else {
                            errors.push(format!(
                                "Enum #{n} '{identifier}' > value #{m} '{val_id}': field 'color' is invalid."
                            ));
                        }
                    } else {
                        errors.push(format!(
                            "Enum #{n} '{identifier}' > value #{m} '{val_id}': field 'color' is invalid."
                        ));
                    }
                }
                None => {
                    errors.push(format!(
                        "Enum #{n} '{identifier}' > value #{m} '{val_id}': field 'color' is missing."
                    ));
                }
            }

            // Check tileRect if present and not null
            if let Some(rect_val) = val_obj.get("tileRect") {
                if !rect_val.is_null() {
                    let is_valid_rect = rect_val.as_object().is_some_and(|r| {
                        r.get("tilesetUid").is_some_and(|v| v.is_number())
                            && r.get("x").is_some_and(|v| v.is_number())
                            && r.get("y").is_some_and(|v| v.is_number())
                            && r.get("w").is_some_and(|v| v.is_number())
                            && r.get("h").is_some_and(|v| v.is_number())
                    });

                    if !is_valid_rect {
                        errors.push(format!(
                            "Enum #{n} '{identifier}' > value #{m} '{val_id}': incomplete tileRect."
                        ));
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let file_data: FleiyaEnumFile = serde_json::from_value(parsed_val)
        .map_err(|e| vec![format!("Schema deserialization error: {e}")])?;

    Ok(migrate_enum_file(file_data))
}

/// Applies forward migrations keyed on version.
pub fn migrate_enum_file(mut file: FleiyaEnumFile) -> FleiyaEnumFile {
    if file.version < CURRENT_ENUM_FILE_VERSION {
        file.version = CURRENT_ENUM_FILE_VERSION;
    }
    file
}

/// Converts project `EnumDef` instances into a `FleiyaEnumFile` representation.
pub fn convert_project_enums_to_file(
    enums: &[EnumDef],
    tilesets: &[TilesetDef],
    enum_tilesets: Option<&HashMap<EnumDefId, TilesetDefId>>,
    image_size_resolver: &dyn Fn(TilesetDefId) -> Option<(u32, u32)>,
) -> FleiyaEnumFile {
    let file_enums = enums
        .iter()
        .map(|e| {
            let icon_tileset_uid = e
                .tileset_id
                .map(|tid| tid.0 as i64)
                .or_else(|| {
                    enum_tilesets
                        .and_then(|m| m.get(&e.id))
                        .map(|tid| tid.0 as i64)
                })
                .or_else(|| {
                    e.values
                        .iter()
                        .find_map(|v| v.tile.map(|t| t.tileset_id.0 as i64))
                });

            let values = e
                .values
                .iter()
                .map(|v| {
                    let color = rgba_to_int(v.color);
                    let tile_rect = v.tile.and_then(|t| {
                        let ts = tilesets.iter().find(|ts| ts.id == t.tileset_id)?;
                        let img_size = image_size_resolver(ts.id).unwrap_or((256, 256));
                        Some(tile_id_to_rect(ts, t.tile_id, img_size))
                    });

                    FleiyaEnumValueDef {
                        id: v.id.clone(),
                        color,
                        tile_rect,
                    }
                })
                .collect();

            FleiyaEnumDef {
                identifier: e.name.clone(),
                uid: e.id.0 as i64,
                icon_tileset_uid,
                values,
                tags: e.tags.clone(),
                external_rel_path: None,
                external_file_checksum: None,
            }
        })
        .collect();

    FleiyaEnumFile {
        format: ENUM_FILE_FORMAT.to_string(),
        version: CURRENT_ENUM_FILE_VERSION,
        enums: file_enums,
    }
}

/// Converts a single `EnumDef` into a single-enum `FleiyaEnumFile`.
pub fn convert_single_enum_to_file(
    enum_def: &EnumDef,
    tilesets: &[TilesetDef],
    icon_tileset: Option<TilesetDefId>,
    image_size_resolver: &dyn Fn(TilesetDefId) -> Option<(u32, u32)>,
) -> FleiyaEnumFile {
    let mut map = HashMap::new();
    if let Some(ts_id) = icon_tileset {
        map.insert(enum_def.id, ts_id);
    }
    convert_project_enums_to_file(
        std::slice::from_ref(enum_def),
        tilesets,
        Some(&map),
        image_size_resolver,
    )
}

/// Converts a `FleiyaEnumDef` back into an `EnumDef`, clearing any bound tiles missing or invalid.
pub fn convert_file_enum_to_def(
    file_enum: &FleiyaEnumDef,
    target_id: EnumDefId,
    project_tilesets: &[TilesetDef],
    image_size_resolver: &dyn Fn(TilesetDefId) -> Option<(u32, u32)>,
) -> EnumDef {
    let values = file_enum
        .values
        .iter()
        .map(|v| {
            let color = int_to_rgba(v.color);
            let tile = v.tile_rect.and_then(|rect| {
                // 1. tilesetUid must exist in project
                let tileset = project_tilesets
                    .iter()
                    .find(|ts| ts.id.0 as i64 == rect.tileset_uid)?;

                // 2. tilesetUid == iconTilesetUid
                if file_enum.icon_tileset_uid != Some(rect.tileset_uid) {
                    return None;
                }

                // 3. bounds check & 4. grid alignment check
                let img_size = image_size_resolver(tileset.id).unwrap_or((256, 256));
                let tile_id = rect_to_tile_id(tileset, &rect, img_size)?;

                Some(TileRef::new(tileset.id, tile_id))
            });

            EnumValueDef {
                id: v.id.clone(),
                name: v.id.clone(),
                color,
                tile,
            }
        })
        .collect();

    let tileset_id = file_enum
        .icon_tileset_uid
        .and_then(|uid| project_tilesets.iter().find(|ts| ts.id.0 as i64 == uid))
        .map(|ts| ts.id);

    EnumDef {
        id: target_id,
        name: file_enum.identifier.clone(),
        tileset_id,
        tags: file_enum.tags.clone(),
        values,
    }
}

/// Computes the list of keys present in `existing_values` that are absent in `new_values`.
pub fn compute_lost_keys(
    existing_values: &[EnumValueDef],
    new_values: &[FleiyaEnumValueDef],
) -> Vec<String> {
    let new_keys: HashSet<&str> = new_values.iter().map(|v| v.id.as_str()).collect();
    existing_values
        .iter()
        .filter(|v| !new_keys.contains(v.id.as_str()))
        .map(|v| v.id.clone())
        .collect()
}

/// Generates a unique enum identifier by appending `_2`, `_3`, etc.
pub fn generate_unique_identifier(base: &str, existing_names: &HashSet<String>) -> String {
    let mut counter = 2;
    loop {
        let candidate = format!("{}_{}", base, counter);
        if !existing_names.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

/// Serializes and writes a `FleiyaEnumFile` to disk atomically (temp file, then rename).
pub fn write_enum_file_atomic(path: &Path, file_data: &FleiyaEnumFile) -> Result<(), IoError> {
    let json_str = serde_json::to_string_pretty(file_data).map_err(|e| {
        tracing::error!(path = %path.display(), error = %e, "JSON formatting failed");
        IoError::Json {
            path: path.to_path_buf(),
            source: e,
        }
    })?;

    let tmp_path = path.with_extension("tmp_fleiyaenum");
    fs::write(&tmp_path, json_str.as_bytes()).map_err(|e| {
        tracing::error!(path = %tmp_path.display(), error = %e, "Failed to write temp enum file");
        IoError::Fs {
            path: tmp_path.clone(),
            source: e,
        }
    })?;

    fs::rename(&tmp_path, path).map_err(|e| {
        tracing::error!(from = %tmp_path.display(), to = %path.display(), error = %e, "Atomic rename failed");
        IoError::Fs {
            path: path.to_path_buf(),
            source: e,
        }
    })
}

/// Reads a `.fleiyaenum` file from disk and validates it.
pub fn read_enum_file(path: &Path) -> Result<FleiyaEnumFile, Vec<String>> {
    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    let content = fs::read_to_string(path)
        .map_err(|e| vec![format!("I/O error reading '{}': {}", path.display(), e)])?;

    validate_enum_file_content(&content, Some(file_name))
}

/// Validates in-memory enum file data before exporting or writing to disk.
pub fn validate_enum_file_data(file_data: &FleiyaEnumFile) -> Result<(), Vec<String>> {
    let json_str = serde_json::to_string_pretty(file_data)
        .map_err(|e| vec![format!("JSON serialization error: {e}")])?;
    validate_enum_file_content(&json_str, None).map(|_| ())
}

/// Checks if an enum file exists on disk.
pub fn enum_file_exists(path: &Path) -> bool {
    path.exists()
}
