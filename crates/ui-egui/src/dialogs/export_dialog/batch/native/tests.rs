use super::{
    AtomicBool, COMMAND, Conversion, Entry, HashSet, Map, Ordering, Path, PhotocraftApp, State, Value, add_paths, command, convert_one, json, matches_filter,
    scan, target_path, tick, wildcard,
};
use egui_kittest::kittest::Queryable;
use photocraft_codecs::{self as codecs, ChannelLayout, EncodeOptions, Format, Image};

fn image(path: &Path) {
    let mut img = Image::from_u8(32, 24, ChannelLayout::Rgb, [45, 100, 160].repeat(32 * 24)).unwrap();
    let mut exif = b"II*\0\x08\0\0\0".to_vec();
    exif.extend_from_slice(&1u16.to_le_bytes());
    exif.extend_from_slice(&0x010fu16.to_le_bytes());
    exif.extend_from_slice(&2u16.to_le_bytes());
    exif.extend_from_slice(&8u32.to_le_bytes());
    exif.extend_from_slice(&26u32.to_le_bytes());
    exif.extend_from_slice(&0u32.to_le_bytes());
    exif.extend_from_slice(b"TestCam\0");
    img.meta.exif = Some(exif);
    img.meta.xmp = Some("<x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"><rdf:Description xmlns:dc=\"http://purl.org/dc/elements/1.1/\" dc:creator=\"Test Author\"/></rdf:RDF></x:xmpmeta>".into());
    std::fs::write(path, codecs::encode(&img, Format::Jpeg, &EncodeOptions::default()).unwrap()).unwrap();
}

fn fields(folder: &Path, format: &str) -> Map<String, Value> {
    let mut f = State::default().fields;
    f.insert("folder".into(), json!(folder));
    f.insert("format".into(), json!(format));
    f
}

fn entry(path: &Path) -> Entry {
    Entry { source: path.to_str().unwrap().into(), enabled: true, status: "Oczekuje".into(), output: String::new(), message: String::new() }
}

#[test]
fn filters_ignore_case_and_match_multiple_masks() {
    for name in ["wakacje.JPG", "portret.PnG", "obraz.JXL"] {
        assert!(matches_filter(Path::new(name), "*.jpg;*.png;*.jxl"));
    }
    assert!(matches_filter(Path::new("DŁUGA NAZWA.JPG"), "długa*.jpg"));
    assert!(matches_filter(Path::new("DŁUGA NAZWA.JPG"), "długa nazwa*.jpg"));
    assert!(matches_filter(Path::new("a.jpg"), "jpg"));
    assert!(matches_filter(Path::new("a.jpg"), ".jpg"));
    assert!(!matches_filter(Path::new("a.jpg"), "*.png"));
    assert!(wildcard("abc.jpg", "a?c.*"));
    assert!(!wildcard("ac.jpg", "a?c.*"));
}

#[test]
fn folder_filter_recursion_and_output_exclusion() {
    let dir = tempfile::tempdir().unwrap();
    let child = dir.path().join("sub");
    let output = dir.path().join("results");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::create_dir_all(&output).unwrap();
    for p in [dir.path().join("A.JPG"), child.join("B.JPG"), output.join("old.jpg")] {
        image(&p);
    }
    std::fs::write(dir.path().join("not-an-image.txt"), b"text").unwrap();
    let cancel = AtomicBool::new(false);
    assert_eq!(scan(dir.path(), "*.jpg", false, &output, &cancel).unwrap().len(), 1);
    assert_eq!(scan(dir.path(), "*.jpg", true, &output, &cancel).unwrap().len(), 2);
    assert!(scan(dir.path(), "*.png", true, &output, &cancel).unwrap().is_empty());
    cancel.store(true, Ordering::Release);
    assert!(scan(dir.path(), "*", true, &output, &cancel).is_err());
}

#[test]
fn additions_deduplicate_and_errors_do_not_partially_change_the_queue() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.jpg");
    image(&file);
    let mut state = State::default();
    assert_eq!(add_paths(&mut state, vec![file.to_str().unwrap().into(), file.to_str().unwrap().into()]).unwrap(), 1);
    let before = state.entries.clone();
    assert!(add_paths(&mut state, vec![dir.path().join("missing.jpg").to_str().unwrap().into()]).is_err());
    assert_eq!(state.entries, before);
}

#[test]
fn source_files_and_previous_results_are_protected() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("same.jpg");
    image(&source);
    let original = std::fs::read(&source).unwrap();
    let sources = HashSet::from([source.canonicalize().unwrap()]);
    assert!(target_path(&source, "overwrite", &sources, &HashSet::new()).is_err());
    assert!(target_path(&source, "skip", &sources, &HashSet::new()).unwrap().is_none());
    let fresh = target_path(&source, "unique", &sources, &HashSet::new()).unwrap().unwrap();
    assert_eq!(fresh.file_name().unwrap(), "same_1.jpg");
    let written = HashSet::from([fresh.clone()]);
    assert!(target_path(&fresh, "overwrite", &sources, &written).is_err());
    assert_eq!(std::fs::read(source).unwrap(), original);
}

#[test]
fn queued_jpeg_jxl_jpeg_roundtrip_preserves_exif_and_xmp() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.jpg");
    image(&source);
    let sources = HashSet::from([source.canonicalize().unwrap()]);
    let cancel = AtomicBool::new(false);
    for lossless in [false, true] {
        let mut f = fields(dir.path(), "jxl");
        f.insert("lossless".into(), json!(lossless));
        let jxl = convert_one(
            &entry(&source),
            1,
            &f,
            &Conversion { options: &Default::default(), sources: &sources, written: &HashSet::new(), cancel: &cancel },
            &|_| {},
        )
        .unwrap();
        let reopened = codecs::decode(&std::fs::read(&jxl.output).unwrap()).unwrap();
        assert!(reopened.meta.exif.unwrap().windows(7).any(|s| s == b"TestCam"));
        assert!(reopened.meta.xmp.unwrap().contains("Test Author"));
        let jpeg = convert_one(
            &entry(Path::new(&jxl.output)),
            1,
            &fields(dir.path(), "jpg"),
            &Conversion { options: &Default::default(), sources: &sources, written: &HashSet::new(), cancel: &cancel },
            &|_| {},
        )
        .unwrap();
        let reopened = codecs::decode(&std::fs::read(jpeg.output).unwrap()).unwrap();
        assert!(reopened.meta.exif.unwrap().windows(7).any(|s| s == b"TestCam"));
        assert!(reopened.meta.xmp.unwrap().contains("Test Author"));
    }
}

#[test]
fn metadata_none_and_cancellation_are_respected() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.jpg");
    image(&source);
    let sources = HashSet::from([source.canonicalize().unwrap()]);
    let mut f = fields(dir.path(), "png");
    f.insert("metadata".into(), json!("none"));
    let cancel = AtomicBool::new(true);
    assert!(
        convert_one(
            &entry(&source),
            1,
            &f,
            &Conversion { options: &Default::default(), sources: &sources, written: &HashSet::new(), cancel: &cancel },
            &|_| {}
        )
        .is_err()
    );
    assert!(!dir.path().join("source.png").exists());
    cancel.store(false, Ordering::Release);
    let out = convert_one(
        &entry(&source),
        1,
        &f,
        &Conversion { options: &Default::default(), sources: &sources, written: &HashSet::new(), cancel: &cancel },
        &|_| {},
    )
    .unwrap();
    let decoded = codecs::decode(&std::fs::read(out.output).unwrap()).unwrap();
    assert!(decoded.meta.exif.is_none());
    assert!(decoded.meta.xmp.is_none());
}

#[test]
fn worker_skips_unchecked_and_continues_after_corrupt_input() {
    let dir = tempfile::tempdir().unwrap();
    let good = dir.path().join("good.jpg");
    image(&good);
    let unchecked = dir.path().join("unchecked.jpg");
    image(&unchecked);
    let broken = dir.path().join("broken.jpg");
    std::fs::write(&broken, b"not an image").unwrap();
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
    let ctx = egui::Context::default();
    command(&mut app, &ctx, &json!({"action":"add","files":[broken,good,unchecked]})).unwrap();
    app.ui.batch_converter.entries.last_mut().unwrap().enabled = false;
    std::fs::remove_file(&unchecked).unwrap();
    command(&mut app, &ctx, &json!({"action":"start","settings":fields(dir.path(), "png")})).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while app.batch_converter_worker.is_some() {
        tick(&mut app, &ctx);
        assert!(std::time::Instant::now() < deadline, "worker did not finish");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(app.ui.batch_converter.entries[0].status, "Błąd");
    assert_eq!(app.ui.batch_converter.entries[1].status, "Gotowe");
    assert_eq!(app.ui.batch_converter.entries[2].status, "Oczekuje");
    assert!(dir.path().join("good.png").is_file());
    assert!(!dir.path().join("unchecked.png").exists());
    assert_eq!(app.ui.batch_converter.progress, 1.0);
    assert!(app.session.active().is_none(), "the queue must not open editor tabs");
}

#[test]
fn malformed_command_settings_are_errors() {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
    let ctx = egui::Context::default();
    assert!(command(&mut app, &ctx, &json!({"action":"add","files":[42]})).is_err());
    assert!(command(&mut app, &ctx, &json!({"action":"start"})).is_err());
    assert!(command(&mut app, &ctx, &json!({"action":"unknown"})).is_err());
    assert!(crate::menus::is_enabled(&app, COMMAND));
}

#[test]
fn resize_and_file_size_limit_use_the_actual_export() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.jpg");
    image(&source);
    let sources = HashSet::from([source.canonicalize().unwrap()]);
    let cancel = AtomicBool::new(false);
    let mut f = fields(dir.path(), "png");
    f.extend(json!({"fullSize":false,"resizeMode":"longEdge","pixels":16}).as_object().unwrap().clone());
    let result = convert_one(
        &entry(&source),
        1,
        &f,
        &Conversion { options: &Default::default(), sources: &sources, written: &HashSet::new(), cancel: &cancel },
        &|_| {},
    )
    .unwrap();
    let reopened = codecs::decode(&std::fs::read(&result.output).unwrap()).unwrap();
    assert_eq!((reopened.width(), reopened.height()), (16, 12));
    let mut limited = fields(dir.path(), "jpg");
    limited.insert("limitKb".into(), json!(1));
    limited.insert("filenameTemplate".into(), json!("limited"));
    // A long XMP payload makes this small image exceed 1 KB independently of pixel compression.
    let mut source_img = codecs::decode(&std::fs::read(&source).unwrap()).unwrap();
    source_img.meta.xmp = Some(format!("<x:xmpmeta xmlns:x=\"adobe:ns:meta/\">{}</x:xmpmeta>", "X".repeat(4096)));
    std::fs::write(&source, codecs::encode(&source_img, Format::Jpeg, &EncodeOptions::default()).unwrap()).unwrap();
    assert!(
        convert_one(
            &entry(&source),
            1,
            &limited,
            &Conversion { options: &Default::default(), sources: &sources, written: &HashSet::new(), cancel: &cancel },
            &|_| {}
        )
        .is_err()
    );
    assert!(!dir.path().join("limited.jpg").exists());
}

#[test]
#[ignore = "Manual offscreen visual QA; set PHOTOCRAFT_BATCH_QA_PNG"]
fn converter_visual_snapshot() {
    let mut h = egui_kittest::Harness::builder()
        .with_accessibility_check(false)
        .with_size(egui::vec2(1366.0, 768.0))
        .with_pixels_per_point(1.0)
        .with_max_steps(64)
        .wgpu()
        .build_eframe(|cc| {
            PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
            let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
            app.ui.batch_converter.visible = true;
            app.ui.batch_converter.fields.insert("folder".into(), json!("/home/darek/Zdjecia/wyniki"));
            for i in 1..=20 {
                app.ui.batch_converter.entries.push(entry(Path::new(&format!("/home/darek/Zdjecia/Zdjecie_{i:03}.jpg"))));
            }
            app
        });
    h.run_steps(12);
    h.render().unwrap().save(std::env::var("PHOTOCRAFT_BATCH_QA_PNG").unwrap_or_else(|_| "batch-converter.png".into())).unwrap();
    let close = h.get_by_label("Zamknij").rect();
    assert!(
        close.left() >= 0.0 && close.top() >= 0.0 && close.right() <= 1366.0 && close.bottom() <= 768.0,
        "Converter footer outside the viewport: {close:?}"
    );
}
