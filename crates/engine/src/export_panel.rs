//! Full export-panel settings shared by native UI, presets and scriptable exports.
//! Resize modes and output-sharpening targets follow the supplied LightCraft reference.
use crate::Session;
use photocraft_color::{ColorMode, PixelFormat};
use photocraft_doc::{Document, Layer, LayerContent, LayerId};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AdvancedExport {
    pub limit_kb: u64,
    pub full_size: bool,
    pub resize_mode: String,
    pub pixels: f64,
    pub resize_height: f64,
    pub dont_enlarge: bool,
    pub resolution: f64,
    pub sharpen: String,
    pub sharpen_amount: String,
    pub remove_location: bool,
    pub watermark: bool,
    pub watermark_text: String,
    pub watermark_size: f64,
    pub watermark_opacity: f64,
    pub watermark_position: String,
    pub watermark_color: String,
    pub filename_template: String,
    pub destination: String,
    pub folder: String,
    pub subfolder: String,
    pub conflict: String,
    pub jxl_effort: String,
    pub jxl_progressive: bool,
    pub hdr: bool,
}
impl Default for AdvancedExport {
    fn default() -> Self {
        Self {
            limit_kb: 0,
            full_size: false,
            resize_mode: "custom".into(),
            pixels: 2048.0,
            resize_height: 2048.0,
            dont_enlarge: false,
            resolution: 0.0,
            sharpen: "none".into(),
            sharpen_amount: "standard".into(),
            remove_location: false,
            watermark: false,
            watermark_text: "PhotoCraft".into(),
            watermark_size: 3.5,
            watermark_opacity: 70.0,
            watermark_position: "bottomRight".into(),
            watermark_color: "#ffffff".into(),
            filename_template: "{name}".into(),
            destination: "ask".into(),
            folder: String::new(),
            subfolder: String::new(),
            conflict: "unique".into(),
            jxl_effort: "normal".into(),
            jxl_progressive: false,
            hdr: false,
        }
    }
}
impl AdvancedExport {
    pub fn from_fields(fields: &Map<String, Value>) -> Result<Self, String> {
        let out: Self = serde_json::from_value(Value::Object(fields.clone())).map_err(|e| format!("Export settings: {e}"))?;
        out.validate()?;
        Ok(out)
    }
    pub fn fields(&self) -> Map<String, Value> {
        serde_json::to_value(self).ok().and_then(|v| v.as_object().cloned()).unwrap_or_default()
    }
    pub fn validate(&self) -> Result<(), String> {
        let valid = |v: &str, choices: &[&str]| choices.contains(&v);
        if !valid(&self.resize_mode, &["custom", "longEdge", "shortEdge", "width", "height", "fit", "dimensions", "megapixels", "percent"]) {
            return Err("Unknown resize mode".into());
        }
        if !self.pixels.is_finite()
            || self.pixels <= 0.0
            || self.pixels > 300_000.0
            || !self.resize_height.is_finite()
            || !(1.0..=300_000.0).contains(&self.resize_height)
        {
            return Err("Resize values must be positive and at most 300000".into());
        }
        if !self.resolution.is_finite() || !(0.0..=9600.0).contains(&self.resolution) || self.limit_kb > 10_000_000 {
            return Err("Resolution or file size limit is out of range".into());
        }
        if !valid(&self.sharpen, &["none", "screen", "matte", "glossy"]) || !valid(&self.sharpen_amount, &["low", "standard", "high"]) {
            return Err("Unknown output sharpening".into());
        }
        if !valid(&self.destination, &["ask", "specificFolder", "sameAsOriginal"]) || !valid(&self.conflict, &["unique", "overwrite", "skip"]) {
            return Err("Unknown destination or existing-file policy".into());
        }
        if !valid(&self.jxl_effort, &["fast", "normal"]) {
            return Err("JPEG XL effort must be fast or normal".into());
        }
        if !self.watermark_size.is_finite()
            || !(0.5..=50.0).contains(&self.watermark_size)
            || !self.watermark_opacity.is_finite()
            || !(0.0..=100.0).contains(&self.watermark_opacity)
        {
            return Err("Watermark size or opacity is out of range".into());
        }
        if !valid(&self.watermark_position, &["topLeft", "top", "topRight", "left", "center", "right", "bottomLeft", "bottom", "bottomRight"]) {
            return Err("Unknown watermark position".into());
        }
        if self.watermark_color.len() != 7 || !self.watermark_color.starts_with('#') || !self.watermark_color.chars().skip(1).all(|c| c.is_ascii_hexdigit()) {
            return Err("Watermark color must be #RRGGBB".into());
        }
        if self.watermark && self.watermark_text.trim().is_empty() {
            return Err("Enter the watermark text".into());
        }
        if self.watermark_text.chars().count() > 1000 || self.filename_template.len() > 512 || self.folder.len() > 4096 || self.subfolder.len() > 1024 {
            return Err("An export text field is too long".into());
        }
        relative_subfolder(&self.subfolder)?;
        Ok(())
    }
    /// None preserves the existing exact-pixel/scale controls for old actions and presets.
    pub fn dimensions(&self, width: f64, height: f64) -> Result<Option<(f64, f64)>, String> {
        self.validate()?;
        if !width.is_finite() || !height.is_finite() || width < 1.0 || height < 1.0 {
            return Err("Invalid source dimensions".into());
        }
        if self.full_size {
            return Ok(Some((width, height)));
        }
        let v = self.pixels;
        let mut scale = match self.resize_mode.as_str() {
            "custom" => return Ok(None),
            "longEdge" => v / width.max(height),
            "shortEdge" => v / width.min(height),
            "width" => v / width,
            "height" => v / height,
            "fit" => (v / width).min(self.resize_height / height),
            "dimensions" => (v.max(self.resize_height) / width.max(height)).min(v.min(self.resize_height) / width.min(height)),
            "megapixels" => (v * 1e6 / (width * height)).sqrt(),
            "percent" => v / 100.0,
            _ => return Err("Invalid resize mode".into()),
        };
        if self.dont_enlarge {
            scale = scale.min(1.0);
        }
        let out = ((width * scale).round().max(1.0), (height * scale).round().max(1.0));
        if out.0 > 300_000.0 || out.1 > 300_000.0 || (scale > 1.0 && out.0 * out.1 > 512e6) {
            return Err("Requested enlargement exceeds the 512 megapixel limit".into());
        }
        Ok(Some(out))
    }
}

pub fn relative_subfolder(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value.trim());
    if path.components().any(|c| !matches!(c, Component::Normal(_) | Component::CurDir)) || value.contains('\\') || value.contains(':') {
        return Err("Subfolder must be relative and cannot contain '..'".into());
    }
    Ok(path.to_path_buf())
}

/// Expand documented filename tags, then remove characters that cannot belong to one filename.
pub fn file_name(template: &str, name: &str, ext: &str, number: usize, width: u32, height: u32) -> Result<String, String> {
    let stem = Path::new(name).file_stem().and_then(|s| s.to_str()).unwrap_or("image");
    let elapsed = web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map_err(|_| "The system date is before 1970")?;
    let now = chrono::DateTime::<chrono::Utc>::from_timestamp(
        i64::try_from(elapsed.as_secs()).map_err(|_| "The system date is out of range")?,
        elapsed.subsec_nanos(),
    )
    .ok_or("The system date is out of range")?;
    let mut text = if template.trim().is_empty() { "{name}".into() } else { template.to_string() };
    for (tag, value) in [
        ("name", stem.to_string()),
        ("number", format!("{number:03}")),
        ("date", now.format("%Y-%m-%d").to_string()),
        ("year", now.format("%Y").to_string()),
        ("month", now.format("%m").to_string()),
        ("day", now.format("%d").to_string()),
        ("format", ext.into()),
        ("width", width.to_string()),
        ("height", height.to_string()),
    ] {
        text = text.replace(&format!("{{{tag}}}"), &value);
    }
    if text.contains(['{', '}']) {
        return Err("Unknown filename tag; use {name}, {number}, {date}, {year}, {month}, {day}, {format}, {width}, {height}".into());
    }
    let text: String = text.chars().map(|c| if c.is_control() || "<>:\"/\\|?*".contains(c) { '_' } else { c }).take(200).collect();
    let text = text.trim().trim_matches('.');
    if text.is_empty() {
        return Err("The file name is empty".into());
    }
    Ok(format!("{text}.{ext}"))
}

pub struct LimitedExport {
    pub bytes: Vec<u8>,
    pub warnings: Vec<String>,
    pub quality: Option<u8>,
}
/// Encode at actual output dimensions. The target is strict: impossible limits return an error.
/// No resizing or conversion from lossless to lossy is done implicitly.
pub fn encode_limited(
    mut encode: impl FnMut(u8) -> Result<(Vec<u8>, Vec<String>), String>,
    quality: u8,
    limit_kb: u64,
    adjustable: bool,
) -> Result<LimitedExport, String> {
    let q = quality.clamp(1, 100);
    let (bytes, warnings) = encode(q)?;
    let limit = limit_kb.checked_mul(1024).ok_or("File-size limit overflow")?;
    if limit == 0 || bytes.len() as u64 <= limit {
        return Ok(LimitedExport { bytes, warnings, quality: adjustable.then_some(q) });
    }
    if !adjustable {
        return Err(format!(
            "The lossless file needs {} KB; the limit is {limit_kb} KB. Select lossy compression or reduce the dimensions.",
            bytes.len().div_ceil(1024)
        ));
    }
    let (bytes, warnings) = encode(1)?;
    if bytes.len() as u64 > limit {
        return Err(format!("The image cannot fit in {limit_kb} KB even at quality 1 ({} KB). Reduce its dimensions or metadata.", bytes.len().div_ceil(1024)));
    }
    let mut best = LimitedExport { bytes, warnings, quality: Some(1) };
    let (mut lo, mut hi) = (2u8, q.saturating_sub(1));
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let (bytes, warnings) = encode(mid)?;
        if bytes.len() as u64 <= limit {
            best = LimitedExport { bytes, warnings, quality: Some(mid) };
            lo = mid.saturating_add(1);
        } else {
            hi = mid.saturating_sub(1);
        }
    }
    Ok(best)
}

/// Remove location-bearing XMP elements/attributes without rewriting other namespaces.
/// Invalid XML fails closed when metadata redaction was requested.
pub fn redact_xmp(text: &str, camera: bool) -> Result<String, String> {
    let parsed = photocraft_io::xml::parse(text, roxmltree::ParsingOptions::default())?;
    let sensitive = |name: &str, ns: Option<&str>| {
        location_key(name) || (camera && (camera_key(name) || ns.is_some_and(|n| n.contains("/exif/") || n.contains("/exifEX/") || n.contains("/aux/"))))
    };
    let mut ranges = Vec::new();
    for node in parsed.descendants().filter(|n| n.is_element()) {
        if sensitive(node.tag_name().name(), node.tag_name().namespace()) {
            ranges.push(node.range());
            continue;
        }
        for attr in node.attributes() {
            if sensitive(attr.name(), attr.namespace()) {
                ranges.push(attr.range());
            }
        }
    }
    ranges.sort_by_key(|r| r.start);
    let mut merged: Vec<std::ops::Range<usize>> = Vec::new();
    for r in ranges {
        if let Some(last) = merged.last_mut()
            && r.start <= last.end
        {
            last.end = last.end.max(r.end);
        } else {
            merged.push(r);
        }
    }
    let mut out = text.to_string();
    for r in merged.into_iter().rev() {
        if out.is_char_boundary(r.start) && out.is_char_boundary(r.end) {
            out.replace_range(r, "");
        } else {
            return Err("Invalid metadata character range".into());
        }
    }
    Ok(out)
}

fn location_key(name: &str) -> bool {
    let lower: String = name.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect();
    lower.contains("gps")
        || lower.contains("latitude")
        || lower.contains("longitude")
        || ["location", "locationcreated", "locationshown", "city", "state", "country", "countrycode", "sublocation"].contains(&lower.as_str())
}
fn camera_key(name: &str) -> bool {
    let lower: String = name.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect();
    lower.contains("camera")
        || lower.contains("serial")
        || lower.contains("lens")
        || ["make", "model", "datetimeoriginal", "exposuretime", "fnumber", "isospeedratings", "iso", "aperture", "shutterspeed"].contains(&lower.as_str())
}

pub fn metadata(doc: &mut Document, policy: &str, remove_location: bool) -> Result<(), String> {
    match policy {
        "none" => {
            doc.metadata.xmp = None;
            doc.metadata.exif = None;
            doc.metadata.text.clear();
        }
        "copyright" => {
            let info = crate::file_cmds::read_file_info(doc.metadata.xmp.as_deref());
            let fields: Map<String, Value> = ["copyright", "copyrightStatus", "copyrightUrl", "author", "authorTitle"]
                .iter()
                .filter_map(|k| info.get(*k).map(|v| ((*k).into(), v.clone())))
                .collect();
            doc.metadata.xmp = Some(Arc::from(crate::file_cmds::write_file_info(None, &Value::Object(fields))));
            doc.metadata.exif = None;
            doc.metadata.text.clear();
        }
        "noCamera" | "exif" | "all" => {
            if policy == "noCamera" {
                doc.metadata.exif = None;
            }
            if remove_location || policy == "noCamera" {
                if let Some(x) = doc.metadata.xmp.as_deref() {
                    doc.metadata.xmp = Some(Arc::from(redact_xmp(x, policy == "noCamera")?));
                }
                if let Some(x) = doc.metadata.exif.as_deref() {
                    doc.metadata.exif = photocraft_codecs::exif_without_location(x).map(Arc::new);
                }
                doc.metadata.text.retain(|(k, _)| !(location_key(k) || policy == "noCamera" && camera_key(k)));
            }
        }
        _ => return Err("Unknown metadata policy".into()),
    }
    Ok(())
}

/// Apply effects to a copy only. Alpha and document bit depth are retained in the flattened copy.
pub fn finish_document(doc: &Document, settings: &AdvancedExport, metadata_policy: &str, preview_scale: f64) -> Result<Document, String> {
    settings.validate()?;
    let mut out = doc.clone();
    if settings.resolution > 0.0 {
        out.resolution_dpi = settings.resolution as f32;
    }
    if settings.sharpen != "none" {
        let surface = photocraft_compose::flatten_to_surface(&out, PixelFormat::new(out.mode, out.depth, true), None);
        out.layers = vec![Layer::new("Export composite", LayerContent::Raster(surface))];
        let mut session = Session::new();
        session.add_document(out, None);
        let (radius, amount) = match settings.sharpen.as_str() {
            "screen" => (1.0, 35.0),
            "matte" => (2.0, 60.0),
            _ => (1.0, 50.0),
        };
        let amount = amount
            * match settings.sharpen_amount.as_str() {
                "low" => 0.6,
                "high" => 1.5,
                _ => 1.0,
            };
        session
            .execute("filter.sharpen.unsharpMask", json!({"amount":amount,"radius":(radius*preview_scale).max(0.1),"threshold":0}))
            .map_err(|e| e.to_string())?;
        out = session.active().map(|s| (*s.doc).clone()).ok_or("No export composite")?;
    }
    if settings.watermark && !settings.watermark_text.trim().is_empty() {
        out = watermark(&out, settings)?;
    }
    metadata(&mut out, metadata_policy, settings.remove_location)?;
    Ok(out)
}

fn watermark(doc: &Document, settings: &AdvancedExport) -> Result<Document, String> {
    let mut session = Session::new();
    session.add_document(doc.clone(), None);
    if doc.mode != ColorMode::Rgb {
        session.execute("image.mode.rgb", json!({})).map_err(|e| e.to_string())?;
    }
    let size_px = (f64::from(doc.size.width.min(doc.size.height)) * settings.watermark_size / 100.0).max(1.0);
    let size_pt = (size_px * 72.0 / f64::from(doc.resolution_dpi.max(1.0))).clamp(0.1, 1296.0);
    let name = Path::new(&doc.name).file_stem().and_then(|s| s.to_str()).unwrap_or(&doc.name);
    let text = settings.watermark_text.replace("{name}", name);
    let result = session
        .execute(
            "type.create",
            json!({"x":0,"y":size_px,"text":text,"size":size_pt,"font":"DejaVu Sans","color":settings.watermark_color,"name":"Export watermark"}),
        )
        .map_err(|e| e.to_string())?;
    let id = LayerId(result.get("layer").and_then(Value::as_u64).ok_or("Watermark layer was not created")?);
    let bounds = session.active().and_then(|s| s.doc.layer(id)).map(photocraft_doc::comps::position_bounds).ok_or("Watermark has no bounds")?;
    let w = f64::from(doc.size.width);
    let h = f64::from(doc.size.height);
    let margin = f64::from(doc.size.width.min(doc.size.height)) * 0.025;
    let pos = settings.watermark_position.as_str();
    let left = if pos.ends_with("Left") || pos == "left" {
        margin
    } else if pos.ends_with("Right") || pos == "right" {
        w - margin - f64::from(bounds.width())
    } else {
        (w - f64::from(bounds.width())) / 2.0
    };
    let top = if pos.starts_with("top") {
        margin
    } else if pos.starts_with("bottom") {
        h - margin - f64::from(bounds.height())
    } else {
        (h - f64::from(bounds.height())) / 2.0
    };
    session
        .execute("layer.translate", json!({"layer":id.0,"dx":(left-f64::from(bounds.x0)).round() as i32,"dy":(top-f64::from(bounds.y0)).round() as i32}))
        .map_err(|e| e.to_string())?;
    session
        .edit("Export watermark opacity", |d, _| {
            if let Some(l) = d.layer_mut(id) {
                l.opacity = (settings.watermark_opacity / 100.0) as f32;
            }
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    session.active().map(|s| (*s.doc).clone()).ok_or_else(|| "Watermark export failed".into())
}

/// HDR is written as float linear sRGB in TIFF/EXR/Radiance, or 16-bit Rec.2020 PQ in JPEG XL.
pub fn hdr_document(doc: &Document, format: &str) -> Result<Document, String> {
    if !matches!(format, "jxl" | "tif" | "exr" | "hdr") {
        return Err("The selected format does not support HDR output".into());
    }
    let mut session = Session::new();
    session.add_document(doc.clone(), None);
    if doc.mode != ColorMode::Rgb {
        session.execute("image.mode.rgb", json!({})).map_err(|e| e.to_string())?;
    }
    session.execute("image.mode.bits32", json!({})).map_err(|e| e.to_string())?;
    if format != "jxl" {
        session.execute("edit.convertToProfile", json!({"profile":"linear-srgb","intent":"relative"})).map_err(|e| e.to_string())?;
        return session.active().map(|s| (*s.doc).clone()).ok_or_else(|| "No HDR document".into());
    }
    let mut out = session.active().map(|s| (*s.doc).clone()).ok_or("No HDR document")?;
    let image = photocraft_io::document_to_image(&out, &mut Vec::new()).map_err(|e| e.to_string())?;
    let source = image.icc.as_deref().and_then(|b| photocraft_cms::Profile::parse(b).ok()).unwrap_or_else(|| photocraft_cms::Builtin::Srgb.profile().clone());
    let linear = photocraft_cms::builtin::cicp_rgb(9, 8, true).ok_or("Linear Rec.2020 profile unavailable")?;
    let transform = photocraft_cms::Transform::new(&source, &linear, photocraft_cms::Intent::RelativeColorimetric, false).map_err(|e| e.to_string())?;
    let channels = image.layout().channels();
    let mut samples = image.to_normalized();
    transform.apply(&mut samples, channels);
    for pixel in samples.chunks_exact_mut(channels) {
        for c in pixel.iter_mut().take(3) {
            let l = ((*c).max(0.0) * 203.0 / 10000.0).clamp(0.0, 1.0).powf(2610.0 / 16384.0);
            *c = ((3424.0 / 4096.0 + (2413.0 / 128.0) * l) / (1.0 + (2392.0 / 128.0) * l)).powf(2523.0 / 32.0);
        }
    }
    out.depth = photocraft_color::SampleType::U16;
    let alpha = image.layout().has_alpha();
    let mut surface = photocraft_raster::Surface::new(PixelFormat::new(ColorMode::Rgb, out.depth, alpha));
    surface.write_region(out.bounds(), &samples);
    out.layers = vec![Layer::new("HDR Rec.2020 PQ", LayerContent::Raster(surface))];
    out.icc_profile = Some(Arc::new(photocraft_codecs::jxl_hdr_profile()?));
    Ok(out)
}
