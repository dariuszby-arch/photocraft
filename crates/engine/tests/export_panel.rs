use photocraft_color::{Color, ColorMode, SampleType};
use photocraft_doc::{Document, Size};
use photocraft_engine::export_panel::{AdvancedExport, encode_limited, file_name, finish_document, hdr_document, metadata, redact_xmp, relative_subfolder};
use serde_json::json;
use std::sync::Arc;

#[test]
fn resize_modes_and_no_enlargement_are_consistent() {
    for (mode, pixels, height, expected) in [
        ("longEdge", 2000., 1000., (2000., 1000.)),
        ("shortEdge", 1000., 1000., (2000., 1000.)),
        ("width", 1200., 1000., (1200., 600.)),
        ("height", 1200., 1000., (2400., 1200.)),
        ("fit", 1200., 400., (800., 400.)),
        ("dimensions", 400., 1200., (800., 400.)),
        ("megapixels", 2., 1000., (2000., 1000.)),
        ("percent", 25., 1000., (1000., 500.)),
    ] {
        let settings = AdvancedExport { resize_mode: mode.into(), pixels, resize_height: height, ..Default::default() };
        assert_eq!(settings.dimensions(4000., 2000.).unwrap(), Some(expected), "{mode}");
    }
    let mut a = AdvancedExport { resize_mode: "longEdge".into(), pixels: 8000., dont_enlarge: true, ..Default::default() };
    assert_eq!(a.dimensions(4000., 2000.).unwrap(), Some((4000., 2000.)));
    a.dont_enlarge = false;
    assert_eq!(a.dimensions(4000., 2000.).unwrap(), Some((8000., 4000.)));
    a.pixels = 300_000.;
    assert!(a.dimensions(4000., 2000.).is_err());
    a.full_size = true;
    assert_eq!(a.dimensions(4000., 2000.).unwrap(), Some((4000., 2000.)));
}

#[test]
fn file_size_limit_is_strict_and_never_changes_lossless_to_lossy() {
    let encode = |q: u8| Ok((vec![q; usize::from(q) * 100], vec![q.to_string()]));
    let best = encode_limited(encode, 90, 4, true).unwrap();
    assert_eq!(best.quality, Some(40));
    assert_eq!(best.bytes.len(), 4000);
    assert_eq!(best.warnings, ["40"]);
    assert!(encode_limited(encode, 90, 4, false).err().unwrap().contains("lossless"));
    assert!(encode_limited(|_| Ok((vec![0; 2048], vec![])), 90, 1, true).err().unwrap().contains("cannot fit"));
    assert_eq!(encode_limited(encode, 90, 0, true).unwrap().bytes.len(), 9000);
}

#[test]
fn metadata_redaction_handles_attributes_elements_and_preserves_rights() {
    let xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><rdf:Description xmlns:exif="http://ns.adobe.com/exif/1.0/" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:aux="http://ns.adobe.com/exif/1.0/aux/" exif:GPSLatitude="51,0N" aux:SerialNumber="secret"><exif:GPSLongitude>12,0E</exif:GPSLongitude><exif:ExposureTime>1/100</exif:ExposureTime><dc:rights>Author ©</dc:rights></rdf:Description></rdf:RDF></x:xmpmeta>"#;
    let gps = redact_xmp(xmp, false).unwrap();
    assert!(!gps.contains("GPS"));
    assert!(gps.contains("ExposureTime") && gps.contains("Author ©"));
    assert!(photocraft_io::xml::parse(&gps, roxmltree::ParsingOptions::default()).is_ok());
    let camera = redact_xmp(xmp, true).unwrap();
    assert!(!camera.contains("ExposureTime") && !camera.contains("secret"));
    assert!(camera.contains("Author ©"));
    assert!(redact_xmp("<broken", false).is_err());
    let mut doc = document(SampleType::U16);
    doc.metadata.xmp = Some(Arc::from(xmp));
    doc.metadata.exif = Some(Arc::new(vec![1, 2, 3]));
    doc.metadata.text.push(("GPS position".into(), "private".into()));
    doc.metadata.text.push(("City".into(), "private".into()));
    doc.metadata.text.push(("Camera model".into(), "private".into()));
    metadata(&mut doc, "noCamera", false).unwrap();
    assert!(doc.metadata.text.is_empty());
    metadata(&mut doc, "none", false).unwrap();
    assert!(doc.metadata.xmp.is_none() && doc.metadata.exif.is_none() && doc.metadata.text.is_empty());
}

#[test]
fn filename_tags_preserve_polish_letters_and_cannot_escape_a_folder() {
    assert_eq!(file_name("{name}_{number}_{width}x{height}", "Zażółć.tif", "jxl", 2, 800, 600).unwrap(), "Zażółć_002_800x600.jxl");
    assert_eq!(file_name("../{name}", "image", "png", 1, 1, 1).unwrap(), "_image.png");
    assert!(file_name("{unknown}", "image", "jxl", 1, 1, 1).is_err());
    for bad in ["../outside", "/absolute", "x/../../outside", "C:\\temp", "x\\..\\y"] {
        assert!(relative_subfolder(bad).is_err(), "{bad}");
    }
    assert!(relative_subfolder("export/2026").is_ok());
}

fn document(depth: SampleType) -> Document {
    Document::with_background("test.tif", Size::new(128, 64), ColorMode::Rgb, depth, Color::BLACK)
}

#[test]
fn watermark_and_resolution_modify_only_the_export_copy_at_each_depth() {
    for depth in [SampleType::U8, SampleType::U16, SampleType::F32] {
        let doc = document(depth);
        let settings = AdvancedExport { watermark: true, watermark_text: "Test ©".into(), watermark_size: 20., resolution: 300., ..Default::default() };
        let out = finish_document(&doc, &settings, "none", 1.).unwrap();
        assert_eq!(out.depth, depth);
        assert_eq!(out.resolution_dpi, 300.);
        assert_eq!(doc.layers.len(), 1);
        assert_ne!(doc.resolution_dpi, 300.);
        assert_eq!(out.layers.len(), 2);
        let flattened = photocraft_compose::flatten_to_surface(&out, out.pixel_format(), None);
        assert!(flattened.read_region(out.bounds()).chunks_exact(4).any(|p| p[0] > 0.05), "{depth:?}");
    }
}

#[test]
fn advanced_presets_roundtrip_and_old_presets_remain_valid() {
    let a = AdvancedExport::from_fields(json!({"limitKb":400,"resizeMode":"longEdge","pixels":2048,"resolution":240,"sharpen":"screen","watermark":true,"watermarkText":"© Fotograf","destination":"specificFolder","folder":"/tmp/export","subfolder":"jxl","conflict":"skip","jxlEffort":"fast","hdr":true}).as_object().unwrap()).unwrap();
    assert_eq!(AdvancedExport::from_fields(&a.fields()).unwrap(), a);
    assert_eq!(AdvancedExport::from_fields(json!({}).as_object().unwrap()).unwrap().dimensions(800., 600.).unwrap(), None);
    assert!(AdvancedExport::from_fields(json!({"watermarkSize":-1}).as_object().unwrap()).is_err());
}

#[test]
fn hdr_profiles_and_depths_are_explicit() {
    let doc = document(SampleType::U16);
    let jxl = hdr_document(&doc, "jxl").unwrap();
    assert_eq!(jxl.depth, SampleType::U16);
    assert!(photocraft_cms::Profile::parse(jxl.icc_profile.as_ref().unwrap()).is_ok());
    let tiff = hdr_document(&doc, "tif").unwrap();
    assert_eq!(tiff.depth, SampleType::F32);
    assert!(hdr_document(&doc, "jpg").is_err());
    assert_eq!(doc.layers.len(), 1);
}

#[test]
fn hdr_jxl_roundtrips_pq_white_in_lossless_and_lossy_modes() {
    let doc = Document::with_background("white", Size::new(32, 16), ColorMode::Rgb, SampleType::U16, Color::WHITE);
    let out = hdr_document(&doc, "jxl").unwrap();
    for lossless in [true, false] {
        let mut opts = photocraft_io::ExportOptions::default();
        opts.encode.jxl.hdr = true;
        opts.encode.jxl.lossless = lossless;
        let bytes = photocraft_io::export(&out, "white.jxl", &opts).unwrap().bytes;
        let imported = photocraft_io::import("white.jxl", &bytes).unwrap().document;
        let rgba = imported.layers[0].surface().unwrap().rgba(0, 0);
        assert!((0.56..0.60).contains(&rgba[0]), "lossless={lossless}, {rgba:?}");
        assert!(photocraft_cms::Profile::parse(imported.icc_profile.as_ref().unwrap()).is_ok());
    }
}

#[test]
fn jxl_explicit_resolution_roundtrips_even_without_descriptive_metadata() {
    let mut doc = document(SampleType::U16);
    doc.resolution_dpi = 240.;
    let bytes = photocraft_io::export(&doc, "resolution.jxl", &Default::default()).unwrap().bytes;
    let imported = photocraft_io::import("resolution.jxl", &bytes).unwrap().document;
    assert_eq!(imported.resolution_dpi, 240.);
}
