//! LightCraft-style controls and native export placement.
use super::{ExportSettings, Map, PhotocraftApp, Value, json, n, s, s_fmt};
use photocraft_doc::Document;
use photocraft_engine::export_panel::{self, AdvancedExport};

pub(super) fn read(f: &Map<String, Value>) -> Result<AdvancedExport, String> {
    AdvancedExport::from_fields(f)
}
pub(super) fn keys() -> Vec<String> {
    AdvancedExport::default().fields().into_iter().map(|(k, _)| k).collect()
}
pub(super) fn valid(k: &str, v: &Value) -> bool {
    let mut f = AdvancedExport::default().fields();
    if !f.contains_key(k) {
        return false;
    }
    f.insert(k.into(), v.clone());
    read(&f).is_ok()
}
pub(super) fn defaults(f: &mut Map<String, Value>, doc: &Document, source: Option<&str>) {
    // Keep Export As's remembered scale and pixel-size behavior. The independent
    // disk-file queue chooses full-size/no-enlargement defaults in its own State.
    let settings = AdvancedExport { resolution: f64::from(doc.resolution_dpi), ..Default::default() };
    f.extend(settings.fields());
    if let Some(path) = source {
        f.insert("__sourcePath".into(), json!(path));
    }
}
pub(super) fn encode(
    doc: &Document,
    f: &Map<String, Value>,
    path: &str,
    mut exporter: impl FnMut(&Document, &str, &ExportSettings) -> Result<(Vec<u8>, Vec<String>), String>,
) -> Result<export_panel::LimitedExport, String> {
    let a = read(f)?;
    let fmt = s_fmt(f);
    let settings = super::settings(f);
    let adjustable = matches!(fmt.as_str(), "jpg" | "avif") || (matches!(fmt.as_str(), "webp" | "jxl") && !super::lossless(f));
    export_panel::encode_limited(
        |q| {
            let mut settings = settings.clone();
            if fmt == "jpg" || fmt == "avif" {
                settings.jpeg_quality = Some(q);
            }
            if fmt == "webp" {
                settings.webp_quality = Some(q);
            }
            if fmt == "jxl" {
                settings.jxl.quality = q;
            }
            if fmt == "dng" {
                return super::linear_dng::encode(doc).map(|b| {
                    let warnings =
                        if doc.metadata.exif.is_some() { vec!["Rendered DNG preserves XMP; original camera EXIF is omitted".into()] } else { Vec::new() };
                    (b, warnings)
                });
            }
            exporter(doc, path, &settings)
        },
        n(f, "quality", 90.0).clamp(1.0, 100.0) as u8,
        a.limit_kb,
        adjustable,
    )
}

fn toggle(ui: &mut egui::Ui, f: &mut Map<String, Value>, key: &str, label: &str, default: bool) {
    let mut v = f.get(key).and_then(Value::as_bool).unwrap_or(default);
    crate::widgets::checkbox(ui, &mut v, label);
    f.insert(key.into(), json!(v));
}
fn text(ui: &mut egui::Ui, f: &mut Map<String, Value>, key: &str, label: &str, width: f32) {
    ui.horizontal(|ui| {
        let caption = ui.label(label);
        let mut v = s(f, key);
        ui.add(egui::TextEdit::singleline(&mut v).id_salt(key).desired_width(width)).labelled_by(caption.id);
        f.insert(key.into(), json!(v));
    });
}
fn number(ui: &mut egui::Ui, f: &mut Map<String, Value>, key: &str, label: &str, default: f64, range: std::ops::RangeInclusive<f32>, suffix: &str) {
    let mut v = n(f, key, default) as f32;
    ui.horizontal(|ui| {
        ui.label(label);
        crate::widgets::value_field(ui, &mut v, range, suffix, 105.0).accessible_name(label);
    });
    f.insert(key.into(), json!(f64::from(v)));
}
fn choices(ui: &mut egui::Ui, f: &mut Map<String, Value>, key: &str, label: &str, options: &[(&str, &str)]) {
    ui.horizontal_wrapped(|ui| {
        ui.label(label);
        let mut value = s(f, key);
        for (k, l) in options {
            if ui.add(egui::Button::selectable(value == *k, *l).wrap_mode(egui::TextWrapMode::Extend)).clicked() {
                value = (*k).into();
            }
        }
        f.insert(key.into(), json!(value));
    });
}
fn file_controls(ui: &mut egui::Ui, f: &mut Map<String, Value>, fmt: &str) {
    let hdr_supported = matches!(fmt, "jxl" | "tif" | "exr" | "hdr");
    ui.add_enabled_ui(hdr_supported, |ui| {
        toggle(ui, f, "hdr", "HDR output", false);
    });
    if !hdr_supported {
        f.insert("hdr".into(), json!(false));
    }
    let hdr = f.get("hdr").and_then(Value::as_bool).unwrap_or(false);
    if hdr || matches!(fmt, "dng" | "exr" | "hdr") {
        ui.label(if fmt == "jxl" { "Color space: Rec.2020 PQ (HDR)" } else { "Color space: linear sRGB" });
    } else if (fmt == "jxl" && !super::lossless(f)) || matches!(fmt, "avif" | "tga" | "ico") {
        f.insert("profile".into(), json!("srgb"));
        ui.label("Color space: sRGB");
    } else {
        choices(
            ui,
            f,
            "profile",
            "Color space",
            &[("document", "Document"), ("srgb", "sRGB"), ("p3", "P3"), ("adobergb", "Adobe RGB"), ("prophoto", "ProPhoto"), ("rec2020", "Rec.2020")],
        );
    }
    ui.add_enabled_ui(!matches!(fmt, "avif" | "tga" | "ico" | "exr" | "hdr" | "dng") && (fmt != "jxl" || super::lossless(f)), |ui| {
        toggle(ui, f, "embedProfile", "Embed ICC profile", true);
    });
    if !matches!(fmt, "jpg" | "hdr" | "dng") {
        toggle(ui, f, "transparency", "Transparency", true);
    }
    if matches!(fmt, "jxl" | "webp") {
        toggle(ui, f, "lossless", "Lossless", true);
    }
    let adjustable = matches!(fmt, "jpg" | "avif") || (matches!(fmt, "jxl" | "webp") && !super::lossless(f));
    ui.add_enabled_ui(adjustable, |ui| {
        let mut q = n(f, "quality", 90.0) as f32;
        crate::widgets::slider_row(ui, "Quality", &mut q, 1.0..=100.0, "%", None);
        f.insert("quality".into(), json!(q.round()));
    });
    let mut limit = n(f, "limitKb", 0.0).clamp(0.0, 10_000_000.0) as f32;
    ui.horizontal(|ui| {
        ui.label("Limit file size (KB, 0 = off)");
        crate::widgets::value_field(ui, &mut limit, 0.0..=10_000_000.0, "KB", 100.0).accessible_name("File size limit");
    });
    f.insert("limitKb".into(), json!(limit.round() as u64));
    if limit > 0.0 {
        ui.small(if adjustable {
            "The highest quality fitting the limit is chosen at export."
        } else {
            "A lossless file must fit the limit; its pixels will not be reduced automatically."
        });
    }
    if fmt == "jxl" {
        choices(ui, f, "jxlEffort", "Encoding effort", &[("fast", "Fast"), ("normal", "Normal / smaller file")]);
        toggle(ui, f, "jxlProgressive", "Progressive JPEG XL", false);
    }
}
fn size_controls(ui: &mut egui::Ui, f: &mut Map<String, Value>, src_w: f64, src_h: f64) {
    toggle(ui, f, "fullSize", "Full size", true);
    let full = f.get("fullSize").and_then(Value::as_bool).unwrap_or(true);
    ui.add_enabled_ui(!full, |ui| {
        let options = vec![
            ("longEdge".into(), "Long edge"),
            ("shortEdge".into(), "Short edge"),
            ("width".into(), "Width"),
            ("height".into(), "Height"),
            ("fit".into(), "Width & height"),
            ("dimensions".into(), "Dimensions"),
            ("megapixels".into(), "Megapixels"),
            ("percent".into(), "Percent"),
            ("custom".into(), "Exact dimensions"),
        ];
        let mut mode = s(f, "resizeMode");
        ui.horizontal(|ui| {
            ui.label("Resize to");
            crate::widgets::dropdown(ui, "export-resize-mode", "Resize mode", &mut mode, &options, 180.0);
        });
        f.insert("resizeMode".into(), json!(mode));
        if s(f, "resizeMode") == "custom" {
            super::size_fields(ui, f, src_w, src_h);
        } else {
            let unit = match s(f, "resizeMode").as_str() {
                "percent" => "%",
                "megapixels" => "MP",
                _ => "px",
            };
            number(ui, f, "pixels", "Value", 2048.0, 0.01..=300_000.0, unit);
            if matches!(s(f, "resizeMode").as_str(), "fit" | "dimensions") {
                number(ui, f, "resizeHeight", "Height", 2048.0, 1.0..=300_000.0, "px");
            }
        }
        toggle(ui, f, "dontEnlarge", "Don't enlarge", true);
        super::resample_field(ui, f);
    });
    number(ui, f, "resolution", "Resolution", 240.0, 1.0..=9600.0, "ppi");
    if let Ok((w, h)) = super::export_pixels(src_w, src_h, f, None) {
        ui.small(format!("Output: {w:.0} × {h:.0} pixels"));
    }
    choices(ui, f, "sharpen", "Sharpen", &[("none", "None"), ("screen", "Screen"), ("matte", "Matte"), ("glossy", "Glossy")]);
    if s(f, "sharpen") != "none" {
        choices(ui, f, "sharpenAmount", "Amount", &[("low", "Low"), ("standard", "Standard"), ("high", "High")]);
    }
}
fn metadata_controls(ui: &mut egui::Ui, f: &mut Map<String, Value>, fmt: &str) {
    choices(ui, f, "metadata", "Metadata", &[("all", "All"), ("noCamera", "No camera"), ("copyright", "Copyright"), ("none", "None")]);
    toggle(ui, f, "removeLocation", "Remove location info", false);
    toggle(ui, f, "watermark", "Watermark", false);
    if f.get("watermark").and_then(Value::as_bool) == Some(true) {
        text(ui, f, "watermarkText", "Text", 295.0);
        number(ui, f, "watermarkSize", "Text size", 3.5, 0.5..=50.0, "% of image");
        number(ui, f, "watermarkOpacity", "Opacity", 70.0, 0.0..=100.0, "%");
        let opts = vec![
            ("topLeft".into(), "Top left"),
            ("top".into(), "Top"),
            ("topRight".into(), "Top right"),
            ("left".into(), "Left"),
            ("center".into(), "Center"),
            ("right".into(), "Right"),
            ("bottomLeft".into(), "Bottom left"),
            ("bottom".into(), "Bottom"),
            ("bottomRight".into(), "Bottom right"),
        ];
        let mut p = s(f, "watermarkPosition");
        ui.horizontal(|ui| {
            ui.label("Position");
            crate::widgets::dropdown(ui, "watermark-position", "Watermark position", &mut p, &opts, 190.0);
        });
        f.insert("watermarkPosition".into(), json!(p));
        text(ui, f, "watermarkColor", "Color (#RRGGBB)", 160.0);
    }
    if fmt == "dng" {
        ui.small("Rendered linear DNG: 16-bit RGB, uncompressed; XMP is preserved when selected.");
    }
    if matches!(fmt, "avif" | "exr" | "hdr" | "tga" | "ico") {
        ui.small("This encoder stores pixels only; descriptive metadata and print resolution are omitted.");
    }
    if fmt == "tga" {
        ui.horizontal(|ui| {
            ui.label("Targa bits");
            let mut bits = s(f, "tgaBits");
            let opts = vec![("auto".into(), "Auto"), ("24".into(), "24"), ("32".into(), "32")];
            crate::widgets::dropdown(ui, "tga-depth", "Bits", &mut bits, &opts, 120.0);
            f.insert("tgaBits".into(), json!(bits));
        });
    }
    if fmt == "psd" || fmt == "tif" {
        ui.small("These formats can retain document layers; output sharpening produces a flattened copy.");
    }
}
fn destination_controls(ui: &mut egui::Ui, f: &mut Map<String, Value>) {
    text(ui, f, "filenameTemplate", "File name", 300.0);
    if ui.small_button("Tags…").clicked() {
        let show = f.get("__showTags").and_then(Value::as_bool).unwrap_or(false);
        f.insert("__showTags".into(), json!(!show));
    }
    if f.get("__showTags").and_then(Value::as_bool) == Some(true) {
        ui.small("{name}  {number}  {date}  {year}  {month}  {day}  {format}  {width}  {height}");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        choices(
            ui,
            f,
            "destination",
            "Export to",
            &[("ask", "Choose file"), ("specificFolder", "Specific folder"), ("sameAsOriginal", "Same folder as original")],
        );
        if s(f, "destination") == "specificFolder" {
            ui.horizontal(|ui| {
                ui.label("Folder");
                let mut folder = s(f, "folder");
                ui.add(egui::TextEdit::singleline(&mut folder).id_salt("export-folder").desired_width(260.0));
                f.insert("folder".into(), json!(folder));
                if ui.button("Choose…").clicked() {
                    f.insert(crate::dialogs::BROWSE_FIELD.into(), json!("folder"));
                }
            });
        }
        if s(f, "destination") != "ask" {
            text(ui, f, "subfolder", "Subfolder", 300.0);
        }
        choices(ui, f, "conflict", "If file exists", &[("unique", "Add number"), ("overwrite", "Overwrite"), ("skip", "Skip")]);
    }
    #[cfg(target_arch = "wasm32")]
    {
        f.insert("destination".into(), json!("ask"));
    }
}
pub(super) fn body(app: &mut PhotocraftApp, ui: &mut egui::Ui, f: &mut Map<String, Value>, src_w: f64, src_h: f64) {
    super::preset_fields(app, ui, f, src_w, src_h);
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        ui.label("Format");
        for (format, label) in super::FORMATS {
            let codec_available = photocraft_codecs::from_extension(format).is_none_or(|fmt| fmt.caps().write);
            let enabled = codec_available
                && (format != "original"
                    || (!s(f, "__sourcePath").is_empty() && !f.contains_key("__layer") && !f.contains_key("__layers") && !cfg!(target_arch = "wasm32")));
            ui.add_enabled_ui(enabled, |ui| {
                if ui.add(egui::Button::selectable(s_fmt(f) == format, label).wrap_mode(egui::TextWrapMode::Extend)).clicked() {
                    super::set_format_defaults(f, format, &app.session.prefs().export);
                }
            });
        }
    });
    let fmt = s_fmt(f);
    let original = fmt == "original";
    if original {
        ui.label("Original file: an unchanged copy of the file on disk.");
    }
    ui.add_enabled_ui(!original, |ui| {
        file_controls(ui, f, &fmt);
        ui.separator();
        size_controls(ui, f, src_w, src_h);
        ui.separator();
        metadata_controls(ui, f, &fmt);
    });
    ui.separator();
    destination_controls(ui, f);
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn batch_controls(app: &PhotocraftApp, ui: &mut egui::Ui, f: &mut Map<String, Value>) {
    ui.label("Format całej kolejki");
    ui.horizontal_wrapped(|ui| {
        for (format, label) in super::FORMATS.into_iter().filter(|(format, _)| *format != "original") {
            let enabled = photocraft_codecs::from_extension(format).is_none_or(|fmt| fmt.caps().write);
            if ui.add_enabled(enabled, egui::Button::selectable(s_fmt(f) == format, label)).clicked() {
                super::set_format_defaults(f, format, &app.session.prefs().export);
            }
        }
    });
    let fmt = s_fmt(f);
    file_controls(ui, f, &fmt);
    ui.separator();
    // There is no single source size in a mixed queue. Each file supplies its own dimensions.
    size_controls(ui, f, 0.0, 0.0);
    ui.separator();
    metadata_controls(ui, f, &fmt);
    ui.separator();
    text(ui, f, "filenameTemplate", "File name", 230.0);
    ui.small("{name}  {number}  {date}  {format}  {width}  {height}");
    choices(ui, f, "conflict", "If file exists", &[("unique", "Add number"), ("overwrite", "Overwrite"), ("skip", "Skip")]);
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn destination(f: &Map<String, Value>, name: &str) -> Result<Option<String>, String> {
    let a = read(f)?;
    let mut folder = match a.destination.as_str() {
        "ask" => return Ok(None),
        "specificFolder" => {
            if a.folder.trim().is_empty() {
                return Err("Choose an export folder".into());
            }
            let p = std::path::PathBuf::from(a.folder.trim());
            if !p.is_absolute() {
                return Err("The export folder must be an absolute path".into());
            }
            p
        }
        "sameAsOriginal" => {
            std::path::Path::new(&s(f, "__sourcePath")).parent().filter(|p| !p.as_os_str().is_empty()).ok_or("The document has no source folder")?.to_path_buf()
        }
        _ => return Err("Unknown export destination".into()),
    };
    folder.push(export_panel::relative_subfolder(&a.subfolder)?);
    Ok(Some(folder.join(name).to_string_lossy().into_owned()))
}
#[cfg(target_arch = "wasm32")]
pub(super) fn destination(_f: &Map<String, Value>, _name: &str) -> Result<Option<String>, String> {
    Ok(None)
}

pub(super) fn conflict_path(f: &Map<String, Value>, path: &str) -> Result<Option<String>, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let a = read(f)?;
        let mut target = std::path::PathBuf::from(path);
        if target.exists() {
            match a.conflict.as_str() {
                "skip" => return Ok(None),
                "unique" => {
                    let stem = target.file_stem().and_then(|s| s.to_str()).unwrap_or("image").to_string();
                    let ext = target.extension().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let parent = target.parent().unwrap_or_else(|| std::path::Path::new("")).to_path_buf();
                    let mut i = 2u64;
                    while target.exists() {
                        target = parent.join(format!("{stem}-{i}.{ext}"));
                        i = i.checked_add(1).ok_or("Too many existing output files")?;
                    }
                }
                "overwrite" => {}
                _ => return Err("Unknown file conflict policy".into()),
            }
        }
        Ok(Some(target.to_string_lossy().into_owned()))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = f;
        Ok(Some(path.into()))
    }
}
