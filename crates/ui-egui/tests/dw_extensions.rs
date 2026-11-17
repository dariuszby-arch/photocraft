//! End-to-end checks of the port through the real public menu and frame loop.
use photocraft_codecs::{self as codecs, ChannelLayout, EncodeOptions, Format, Image};
use photocraft_ui_egui::{PhotocraftApp, Services, menus};
use serde_json::json;
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn fixture(path: &Path) -> Vec<u8> {
    let mut image = Image::from_u8(32, 24, ChannelLayout::Rgb, [45, 100, 160].repeat(32 * 24)).unwrap();
    let mut exif = b"II*\0\x08\0\0\0".to_vec();
    exif.extend_from_slice(&1u16.to_le_bytes());
    exif.extend_from_slice(&0x010fu16.to_le_bytes());
    exif.extend_from_slice(&2u16.to_le_bytes());
    exif.extend_from_slice(&8u32.to_le_bytes());
    exif.extend_from_slice(&26u32.to_le_bytes());
    exif.extend_from_slice(&0u32.to_le_bytes());
    exif.extend_from_slice(b"TestCam\0");
    image.meta.exif = Some(exif);
    image.meta.xmp = Some("<x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"><rdf:Description xmlns:dc=\"http://purl.org/dc/elements/1.1/\" dc:creator=\"DW Test Author\"/></rdf:RDF></x:xmpmeta>".into());
    let bytes = codecs::encode(&image, Format::Jpeg, &EncodeOptions::default()).unwrap();
    std::fs::write(path, &bytes).unwrap();
    bytes
}

#[test]
fn batch_menu_continues_after_errors_and_preserves_metadata_in_both_directions() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("Zażółć.jpg");
    let unchecked = directory.path().join("unchecked.jpg");
    let broken = directory.path().join("broken.jpg");
    let output = directory.path().join("wyniki");
    let original = fixture(&source);
    fixture(&unchecked);
    std::fs::write(&broken, b"This is not a JPEG").unwrap();
    let mut harness = egui_kittest::Harness::builder().with_accessibility_check(false).with_size(egui::vec2(1366.0, 768.0)).build_eframe(|cc| {
        PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
        PhotocraftApp::new(photocraft_engine::Session::new(), Services::default())
    });
    let ctx = harness.ctx.clone();
    menus::invoke(harness.state_mut(), &ctx, "file.export.batchConverter", json!({"action":"add", "files":[broken,source,unchecked]})).unwrap();
    {
        let state = &mut harness.state_mut().ui.batch_converter;
        assert_eq!(state.entries.len(), 3);
        state.entries[2].enabled = false;
        state.fields.insert("folder".into(), json!(output));
        state.fields.insert("lossless".into(), json!(true));
        state.fields.insert("metadata".into(), json!("all"));
    }
    menus::invoke(harness.state_mut(), &ctx, "file.export.batchConverter", json!({"action":"start"})).unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while harness.state().ui.batch_converter.running {
        assert!(Instant::now() < deadline, "batch conversion timed out");
        harness.run_steps(1);
        std::thread::sleep(Duration::from_millis(10));
    }
    let state = &harness.state().ui.batch_converter;
    assert_eq!(state.entries[0].status, "Błąd");
    assert_eq!(state.entries[1].status, "Gotowe", "{}", state.entries[1].message);
    assert_eq!(state.entries[2].status, "Oczekuje");
    let jxl = state.entries[1].output.clone();
    let decoded = codecs::decode(&std::fs::read(&jxl).unwrap()).unwrap();
    assert_eq!(decoded.dimensions(), (32, 24));
    assert!(decoded.meta.exif.unwrap().windows(7).any(|s| s == b"TestCam"));
    assert!(decoded.meta.xmp.unwrap().contains("DW Test Author"));
    assert_eq!(std::fs::read(&source).unwrap(), original);
    harness.state_mut().ui.batch_converter.entries.clear();
    harness.state_mut().ui.batch_converter.fields.insert("format".into(), json!("jpg"));
    menus::invoke(harness.state_mut(), &ctx, "file.export.batchConverter", json!({"action":"add","files":[jxl]})).unwrap();
    menus::invoke(harness.state_mut(), &ctx, "file.export.batchConverter", json!({"action":"start"})).unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while harness.state().ui.batch_converter.running {
        assert!(Instant::now() < deadline, "reverse conversion timed out");
        harness.run_steps(1);
        std::thread::sleep(Duration::from_millis(10));
    }
    let row = &harness.state().ui.batch_converter.entries[0];
    assert_eq!(row.status, "Gotowe", "{}", row.message);
    let decoded = codecs::decode(&std::fs::read(&row.output).unwrap()).unwrap();
    assert!(decoded.meta.exif.unwrap().windows(7).any(|s| s == b"TestCam"));
    assert!(decoded.meta.xmp.unwrap().contains("DW Test Author"));
    assert_eq!(std::fs::read(source).unwrap(), original);
}

#[test]
#[ignore = "Offscreen visual acceptance; set PHOTOCRAFT_DW_QA_DIR"]
fn converter_and_export_panel_render_offscreen() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("test.jpg");
    fixture(&source);
    let mut harness = egui_kittest::Harness::builder()
        .with_accessibility_check(false)
        .with_size(egui::vec2(1366.0, 768.0))
        .with_pixels_per_point(1.0)
        .with_max_steps(64)
        .wgpu()
        .build_eframe(|cc| {
            PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
            let services = Services {
                export: Some(Box::new(|doc, path, settings| {
                    photocraft_io::export(doc, path, &settings.io_options()).map(|result| (result.bytes, result.warnings)).map_err(|error| error.to_string())
                })),
                ..Default::default()
            };
            PhotocraftApp::new(photocraft_engine::Session::new(), services)
        });
    let ctx = harness.ctx.clone();
    menus::invoke(harness.state_mut(), &ctx, "file.export.batchConverter", json!({"action":"add","files":[source]})).unwrap();
    harness.state_mut().ui.batch_converter.fields.insert("folder".into(), json!(directory.path().join("wyniki")));
    harness.run_steps(12);
    let out = std::env::var_os("PHOTOCRAFT_DW_QA_DIR").map(std::path::PathBuf::from).expect("set QA output directory");
    std::fs::create_dir_all(&out).unwrap();
    harness.render().unwrap().save(out.join("batch-converter.png")).unwrap();
    harness.state_mut().ui.batch_converter.visible = false;
    harness.state_mut().run("file.new", json!({"width":640,"height":480,"background":"white"})).unwrap();
    menus::invoke(harness.state_mut(), &ctx, "file.export.exportAs", json!({})).unwrap();
    if let Some(dialog) = harness.state_mut().ui.dialogs.last_mut() {
        dialog.fields.insert("format".into(), json!("jxl"));
        dialog.fields.insert("lossless".into(), json!(false));
        dialog.fields.insert("quality".into(), json!(85));
        dialog.fields.insert("metadata".into(), json!("all"));
    }
    harness.run_steps(12);
    harness.render().unwrap().save(out.join("export-panel.png")).unwrap();
}
