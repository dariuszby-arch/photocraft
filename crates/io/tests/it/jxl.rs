//! Document-level JPEG XL checks: source ICC, depth and lossy colour conversion.
use photocraft_codecs::{self as codecs, ChannelLayout, EncodeOptions, Format, Image};
use photocraft_io::{ExportOptions, export, import};

fn camera_exif() -> Vec<u8> {
    let mut exif = b"II*\0\x08\0\0\0".to_vec();
    exif.extend_from_slice(&2u16.to_le_bytes());
    exif.extend_from_slice(&0x0112u16.to_le_bytes());
    exif.extend_from_slice(&3u16.to_le_bytes());
    exif.extend_from_slice(&1u32.to_le_bytes());
    exif.extend_from_slice(&1u32.to_le_bytes());
    exif.extend_from_slice(&0x010fu16.to_le_bytes());
    exif.extend_from_slice(&2u16.to_le_bytes());
    exif.extend_from_slice(&8u32.to_le_bytes());
    exif.extend_from_slice(&38u32.to_le_bytes());
    exif.extend_from_slice(&0u32.to_le_bytes());
    exif.extend_from_slice(b"TestCam\0");
    exif
}

fn assert_camera_exif(exif: &[u8]) {
    assert_eq!(&exif[..4], b"II*\0");
    let ifd = u32::from_le_bytes(exif[4..8].try_into().unwrap()) as usize;
    let count = u16::from_le_bytes(exif[ifd..ifd + 2].try_into().unwrap()) as usize;
    let make = exif[ifd + 2..ifd + 2 + count * 12].as_chunks::<12>().0.iter().find(|e| u16::from_le_bytes(e[..2].try_into().unwrap()) == 0x010f).unwrap();
    let length = u32::from_le_bytes(make[4..8].try_into().unwrap()) as usize;
    let offset = u32::from_le_bytes(make[8..12].try_into().unwrap()) as usize;
    assert_eq!(&exif[offset..offset + length], b"TestCam\0");
    assert_eq!(codecs::exif_orientation(exif), 1);
}

#[test]
fn jpeg_and_jxl_preserve_exif_and_xmp_in_both_directions() {
    let exif = camera_exif();
    let xmp = "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"><rdf:Description xmlns:dc=\"http://purl.org/dc/elements/1.1/\" dc:creator=\"Test Author\"/></rdf:RDF></x:xmpmeta>";
    let mut image = Image::from_u8(32, 24, ChannelLayout::Rgb, [45, 100, 160].repeat(32 * 24)).unwrap();
    image.meta.exif = Some(exif.clone());
    image.meta.xmp = Some(xmp.into());
    let jpeg = codecs::encode(&image, Format::Jpeg, &EncodeOptions::default()).unwrap();
    let original = import("source.jpg", &jpeg).unwrap().document;
    for lossy in [false, true] {
        let mut options = ExportOptions::default();
        options.encode.jxl.lossless = !lossy;
        options.encode.jxl.quality = 80;
        let encoded = export(&original, "converted.jxl", &options).unwrap();
        let reopened = import("converted.jxl", &encoded.bytes).unwrap().document;
        assert_camera_exif(reopened.metadata.exif.as_deref().unwrap());
        assert_eq!(reopened.metadata.xmp.as_deref(), Some(xmp));
        let jpeg = export(&reopened, "converted.jpg", &ExportOptions::default()).unwrap();
        let result = codecs::decode(&jpeg.bytes).unwrap();
        assert_camera_exif(result.meta.exif.as_deref().unwrap());
        assert_eq!(result.meta.xmp.as_deref(), Some(xmp));
    }
}

#[test]
fn jxl_document_roundtrip_keeps_16bit_and_embedded_profile() {
    let mut src = Image::from_u16(19, 11, ChannelLayout::Rgb, &(0..19 * 11 * 3).map(|i| (i * 97 % 65536) as u16).collect::<Vec<_>>()).unwrap();
    let icc = photocraft_cms::Builtin::DisplayP3.profile().to_bytes().to_vec();
    src.icc = Some(icc.clone());
    src.meta.xmp = Some("<rdf:Description exif:ColorSpace=\"65535\" dc:creator=\"P3 Camera\"/>".into());
    let png = codecs::encode(&src, Format::Png, &EncodeOptions::default()).unwrap();
    let document = import("test.png", &png).unwrap().document;
    let jxl = export(&document, "test.JXL", &ExportOptions::default()).unwrap();
    let decoded = codecs::decode(&jxl.bytes).unwrap();
    assert_eq!(decoded.sample_type(), codecs::SampleType::U16);
    assert_eq!(decoded.data(), src.data());
    assert_eq!(decoded.icc.as_deref(), Some(icc.as_slice()));
    assert_eq!(decoded.meta.xmp, src.meta.xmp, "tagged lossless output retains source colour metadata");
    let reopened = import("test.JXL", &jxl.bytes).unwrap();
    assert!(!reopened.source_read_only);
    assert_eq!(reopened.document.depth, photocraft_color::SampleType::U16);
}

#[test]
fn jxl_lossy_document_export_converts_p3_to_srgb() {
    let mut src = Image::from_u8(32, 32, ChannelLayout::Rgb, [120, 90, 60].repeat(32 * 32)).unwrap();
    src.icc = Some(photocraft_cms::Builtin::DisplayP3.profile().to_bytes().to_vec());
    let png = codecs::encode(&src, Format::Png, &EncodeOptions::default()).unwrap();
    let document = import("p3.png", &png).unwrap().document;
    let mut opts = ExportOptions::default();
    opts.encode.jxl.lossless = false;
    opts.encode.jxl.quality = 95;
    let jxl = export(&document, "p3.jxl", &opts).unwrap();
    let out = codecs::decode(&jxl.bytes).unwrap();
    assert_eq!(out.dimensions(), (32, 32));
    assert!(jxl.warnings.iter().any(|w| w.contains("sRGB")));
    let profile = photocraft_cms::Profile::parse(out.icc.as_deref().unwrap()).unwrap();
    assert!(profile.same_colors(photocraft_cms::Builtin::Srgb.profile()));
}

/// JPEG XL permits ancillary EXIF/XMP boxes after the complete codestream.
/// Reproduces photos that open correctly but lose camera metadata on JPG export.
#[test]
fn trailing_jxl_exif_and_xmp_survive_export_to_jpeg() {
    fn add_box(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&u32::try_from(data.len() + 8).unwrap().to_be_bytes());
        out.extend_from_slice(kind);
        out.extend_from_slice(data);
    }

    fn codestream(bytes: &[u8]) -> Vec<u8> {
        if bytes.starts_with(b"\xff\x0a") {
            return bytes.to_vec();
        }
        let mut out = Vec::new();
        let mut at = 12;
        while at + 8 <= bytes.len() {
            let len = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
            let end = if len == 0 { bytes.len() } else { at + len };
            match &bytes[at + 4..at + 8] {
                b"jxlc" => out.extend_from_slice(&bytes[at + 8..end]),
                b"jxlp" => out.extend_from_slice(&bytes[at + 12..end]),
                _ => {}
            }
            at = end;
        }
        assert!(out.starts_with(b"\xff\x0a"));
        out
    }

    let pixels = Image::from_u8(16, 12, ChannelLayout::Rgb, [45, 100, 160].repeat(16 * 12)).unwrap();
    let encoded = codecs::encode(&pixels, Format::Jxl, &EncodeOptions::default()).unwrap();
    let raw = codestream(&encoded);

    // Minimal valid TIFF/EXIF with an orientation tag and a camera Make string.
    let mut exif = b"II*\0\x08\0\0\0".to_vec();
    exif.extend_from_slice(&2u16.to_le_bytes());
    exif.extend_from_slice(&0x0112u16.to_le_bytes()); // Orientation
    exif.extend_from_slice(&3u16.to_le_bytes());
    exif.extend_from_slice(&1u32.to_le_bytes());
    exif.extend_from_slice(&1u32.to_le_bytes()); // Upright
    exif.extend_from_slice(&0x010fu16.to_le_bytes()); // Make
    exif.extend_from_slice(&2u16.to_le_bytes()); // ASCII
    exif.extend_from_slice(&8u32.to_le_bytes()); // 8 bytes (with trailing NUL)
    exif.extend_from_slice(&38u32.to_le_bytes()); // 8 + 2 + 2*12 + 4
    exif.extend_from_slice(&0u32.to_le_bytes()); // Next IFD
    exif.extend_from_slice(b"TestCam\0");
    let xmp = b"<xmpmeta>source-photo</xmpmeta>";

    let mut jxl = b"\0\0\0\x0cJXL \r\n\x87\n".to_vec();
    add_box(&mut jxl, b"ftyp", b"jxl \0\0\0\0jxl ");
    add_box(&mut jxl, b"jxlc", &raw);
    // Put metadata beyond the next 4 KiB decoder feed. Before the fix,
    // is_loading_done() stopped reading before reaching these boxes.
    add_box(&mut jxl, b"free", &vec![0; 8192]);
    let mut exif_box = vec![0; 4]; // JXL Exif: big-endian TIFF header offset
    exif_box.extend_from_slice(&exif);
    add_box(&mut jxl, b"Exif", &exif_box);
    add_box(&mut jxl, b"xml ", xmp);

    let opened = import("camera.jxl", &jxl).unwrap().document;
    assert_eq!(opened.metadata.exif.as_deref().map(Vec::as_slice), Some(exif.as_slice()));
    assert_eq!(opened.metadata.xmp.as_deref(), Some(std::str::from_utf8(xmp).unwrap()));

    let jpeg = export(&opened, "export.jpg", &ExportOptions::default()).unwrap();
    let result = codecs::decode(&jpeg.bytes).unwrap();
    assert_eq!(result.meta.exif.as_deref(), Some(exif.as_slice()));
    assert_eq!(result.meta.xmp.as_deref(), Some(std::str::from_utf8(xmp).unwrap()));
}

#[test]
fn jxl_lossless_without_icc_converts_p3_samples_to_srgb() {
    let mut src = Image::from_u8(16, 16, ChannelLayout::Rgb, [120, 90, 60].repeat(256)).unwrap();
    src.icc = Some(photocraft_cms::Builtin::DisplayP3.profile().to_bytes().to_vec());
    let png = codecs::encode(&src, Format::Png, &EncodeOptions::default()).unwrap();
    let document = import("p3.png", &png).unwrap().document;
    let mut opts = ExportOptions::default();
    opts.encode.embed_icc = false;
    let jxl = export(&document, "untagged.jxl", &opts).unwrap();
    let out = codecs::decode(&jxl.bytes).unwrap();
    assert_ne!(out.data(), src.data(), "P3 values must be converted before omitting their profile");
    let profile = photocraft_cms::Profile::parse(out.icc.as_deref().unwrap()).unwrap();
    assert!(profile.same_colors(photocraft_cms::Builtin::Srgb.profile()));
    assert!(jxl.warnings.iter().any(|w| w.contains("sRGB")));
}
