//! Rendered linear DNG: 16-bit linear sRGB camera samples, uncompressed, D65 calibration.
//! Adobe DNG 1.4 permits Deflate for floating point/32-bit integers, not 16-bit integers.
//! The original RAW is never rewritten.
use photocraft_doc::Document;
use serde_json::json;

struct Entry {
    tag: u16,
    kind: u16,
    count: u32,
    data: Vec<u8>,
}
fn short(tag: u16, value: u16) -> Entry {
    Entry { tag, kind: 3, count: 1, data: value.to_le_bytes().to_vec() }
}
fn long(tag: u16, value: u32) -> Entry {
    Entry { tag, kind: 4, count: 1, data: value.to_le_bytes().to_vec() }
}
fn bytes(tag: u16, data: Vec<u8>) -> Entry {
    Entry { tag, kind: 1, count: data.len() as u32, data }
}
fn rationals(tag: u16, values: &[(i32, i32)], signed: bool) -> Entry {
    Entry {
        tag,
        kind: if signed { 10 } else { 5 },
        count: values.len() as u32,
        data: values.iter().flat_map(|(a, b)| a.to_le_bytes().into_iter().chain(b.to_le_bytes())).collect(),
    }
}

pub(super) fn encode(doc: &Document) -> Result<Vec<u8>, String> {
    let mut session = photocraft_engine::Session::new();
    session.add_document(doc.clone(), None);
    if doc.mode != photocraft_color::ColorMode::Rgb {
        session.execute("image.mode.rgb", json!({})).map_err(|e| e.to_string())?;
    }
    session.execute("image.mode.bits32", json!({})).map_err(|e| e.to_string())?;
    session.execute("edit.convertToProfile", json!({"profile":"linear-srgb","intent":"relative"})).map_err(|e| e.to_string())?;
    let doc = session.active().ok_or("No rendered DNG image")?;
    let image = photocraft_io::document_to_image(&doc.doc, &mut Vec::new()).map_err(|e| e.to_string())?;
    let n = (image.width() as usize).checked_mul(image.height() as usize).and_then(|n| n.checked_mul(6)).ok_or("DNG is too large")?;
    if n > u32::MAX as usize {
        return Err("Rendered DNG exceeds the classic TIFF size limit".into());
    }
    let mut raw = Vec::new();
    raw.try_reserve_exact(n).map_err(|_| "Cannot allocate DNG samples")?;
    for pixel in image.to_normalized().chunks_exact(image.layout().channels()) {
        let alpha = if image.layout().has_alpha() { pixel.last().copied().unwrap_or(1.0) } else { 1.0 };
        for value in pixel.iter().take(3) {
            let v = ((*value * alpha + (1.0 - alpha)).clamp(0.0, 1.0) * 65535.0).round() as u16;
            raw.extend_from_slice(&v.to_le_bytes());
        }
    }
    let samples = raw;
    let mut entries = vec![
        long(256, image.width()),
        long(257, image.height()),
        Entry { tag: 258, kind: 3, count: 3, data: [16u16, 16, 16].into_iter().flat_map(u16::to_le_bytes).collect() },
        short(259, 1),
        short(262, 34892),
        long(273, 0),
        short(274, 1),
        short(277, 3),
        long(278, image.height()),
        long(279, u32::try_from(samples.len()).map_err(|_| "DNG is too large")?),
        rationals(282, &[((doc.doc.resolution_dpi * 1000.0).round().clamp(1.0, i32::MAX as f32) as i32, 1000)], false),
        rationals(283, &[((doc.doc.resolution_dpi * 1000.0).round().clamp(1.0, i32::MAX as f32) as i32, 1000)], false),
        short(284, 1),
        short(296, 2),
        short(339, 1),
        bytes(50706, vec![1, 4, 0, 0]),
        bytes(50707, vec![1, 3, 0, 0]),
        Entry { tag: 50708, kind: 2, count: 24, data: b"PhotoCraft rendered DNG\0".to_vec() },
        long(50717, 65535),
        short(50778, 21),
        rationals(50728, &[(1, 1), (1, 1), (1, 1)], false),
        rationals(
            50721,
            &[
                (3240454, 1000000),
                (-1537139, 1000000),
                (-498531, 1000000),
                (-969266, 1000000),
                (1876011, 1000000),
                (41556, 1000000),
                (55643, 1000000),
                (-204026, 1000000),
                (1057225, 1000000),
            ],
            true,
        ),
    ];
    if let Some(xmp) = doc.doc.metadata.xmp.as_deref() {
        entries.push(bytes(700, xmp.as_bytes().to_vec()));
    }
    entries.sort_by_key(|e| e.tag);
    let head = 8usize + 2 + entries.len() * 12 + 4;
    let mut offsets = Vec::new();
    let mut extra = Vec::new();
    for e in &entries {
        if e.data.len() > 4 {
            if extra.len() % 2 != 0 {
                extra.push(0);
            }
            offsets.push(u32::try_from(head + extra.len()).map_err(|_| "DNG header is too large")?);
            extra.extend_from_slice(&e.data);
        } else {
            offsets.push(0);
        }
    }
    if extra.len() % 2 != 0 {
        extra.push(0);
    }
    let start = u32::try_from(head + extra.len()).map_err(|_| "DNG header is too large")?;
    let mut out = b"II*\0\x08\0\0\0".to_vec();
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    for (e, offset) in entries.iter().zip(offsets) {
        out.extend_from_slice(&e.tag.to_le_bytes());
        out.extend_from_slice(&e.kind.to_le_bytes());
        out.extend_from_slice(&e.count.to_le_bytes());
        if e.tag == 273 {
            out.extend_from_slice(&start.to_le_bytes());
        } else if e.data.len() > 4 {
            out.extend_from_slice(&offset.to_le_bytes());
        } else {
            out.extend_from_slice(&e.data);
            out.resize(out.len() + 4 - e.data.len(), 0);
        }
    }
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&extra);
    out.extend_from_slice(&samples);
    Ok(out)
}
