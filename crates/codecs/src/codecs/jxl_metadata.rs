//! Shared JPEG XL container and colour metadata handling.
#[cfg(any(not(feature = "jxl"), target_arch = "wasm32", test))]
use crate::{CodecError, Format};
#[cfg(any(not(feature = "jxl"), target_arch = "wasm32", test))]
const F: Format = Format::Jxl;
/// A JPEG XL codestream exported here is labelled sRGB. Rewrite only known,
/// well-formed XMP colour-space declarations when they refer to the source
/// profile. Unrelated metadata (camera, dates, GPS, copyright, etc.) stays
/// byte-for-byte unchanged. Unrecognized XML forms are left untouched rather
/// than risking damage to a valid XMP packet.
pub(super) fn srgb_xmp_metadata(xmp: &str) -> std::borrow::Cow<'_, str> {
    let mut out = xmp.to_owned();
    // Exact attribute/element values only; do not replace free-form user text.
    for (old, new) in [
        ("exif:ColorSpace=\"65535\"", "exif:ColorSpace=\"1\""),
        ("exif:ColorSpace=\"2\"", "exif:ColorSpace=\"1\""),
        ("exif:ColorSpace='65535'", "exif:ColorSpace='1'"),
        ("exif:ColorSpace='2'", "exif:ColorSpace='1'"),
        ("<exif:ColorSpace>65535</exif:ColorSpace>", "<exif:ColorSpace>1</exif:ColorSpace>"),
        ("<exif:ColorSpace>2</exif:ColorSpace>", "<exif:ColorSpace>1</exif:ColorSpace>"),
        ("exif:InteroperabilityIndex=\"R03\"", "exif:InteroperabilityIndex=\"R98\""),
        ("exif:InteroperabilityIndex='R03'", "exif:InteroperabilityIndex='R98'"),
        ("<exif:InteroperabilityIndex>R03</exif:InteroperabilityIndex>", "<exif:InteroperabilityIndex>R98</exif:InteroperabilityIndex>"),
        ("photoshop:ICCProfile=\"Adobe RGB (1998)\"", "photoshop:ICCProfile=\"sRGB\""),
        ("photoshop:ICCProfile='Adobe RGB (1998)'", "photoshop:ICCProfile='sRGB'"),
        ("<photoshop:ICCProfile>Adobe RGB (1998)</photoshop:ICCProfile>", "<photoshop:ICCProfile>sRGB</photoshop:ICCProfile>"),
    ] {
        if out.contains(old) {
            out = out.replace(old, new);
        }
    }
    if out == xmp { std::borrow::Cow::Borrowed(xmp) } else { std::borrow::Cow::Owned(out) }
}

// Preserve the bare codestream when embedding metadata is not requested.
// JPEG XL stores EXIF and XMP in ISO 18181-2 container boxes.
#[cfg(any(not(feature = "jxl"), target_arch = "wasm32", test))]
fn append_jxl_box(out: &mut Vec<u8>, name: &[u8; 4], payload: &[u8]) -> Result<(), CodecError> {
    let short_len = payload.len().checked_add(8).ok_or_else(|| CodecError::encode(F, "JPEG XL box size overflow"))?;
    if let Ok(len) = u32::try_from(short_len) {
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(name);
    } else {
        let long_len = u64::try_from(payload.len()).ok().and_then(|n| n.checked_add(16)).ok_or_else(|| CodecError::encode(F, "JPEG XL box size overflow"))?;
        out.extend_from_slice(&1u32.to_be_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(&long_len.to_be_bytes());
    }
    out.extend_from_slice(payload);
    Ok(())
}

#[cfg(any(not(feature = "jxl"), target_arch = "wasm32", test))]
pub(super) fn jxl_with_metadata(codestream: Vec<u8>, exif: Option<&[u8]>, xmp: Option<&str>) -> Result<Vec<u8>, CodecError> {
    let exif = exif.filter(|b| !b.is_empty());
    let xmp = xmp.filter(|s| !s.is_empty());
    if exif.is_none() && xmp.is_none() {
        return Ok(codestream);
    }
    if !codestream.starts_with(&[0xff, 0x0a]) {
        return Err(CodecError::encode(F, "expected a JPEG XL codestream"));
    }

    let mut output = Vec::new();
    output.extend_from_slice(b"\0\0\0\x0cJXL \r\n\x87\n");
    output.extend_from_slice(b"\0\0\0\x14ftypjxl \0\0\0\0jxl ");
    if let Some(exif) = exif {
        // TIFF data follows a four-byte big-endian offset of zero.
        let mut payload = Vec::new();
        payload.extend_from_slice(&0u32.to_be_bytes());
        payload.extend_from_slice(exif);
        append_jxl_box(&mut output, b"Exif", &payload)?;
    }
    if let Some(xmp) = xmp {
        append_jxl_box(&mut output, b"xml ", xmp.as_bytes())?;
    }
    append_jxl_box(&mut output, b"jxlc", &codestream)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xmp_colour_metadata_is_reconciled_without_changing_unrelated_fields() {
        let source = "<rdf:Description exif:ColorSpace=\"2\" photoshop:ICCProfile=\"Adobe RGB (1998)\" dc:creator=\"Camera A\"><exif:InteroperabilityIndex>R03</exif:InteroperabilityIndex></rdf:Description>";
        let adjusted = srgb_xmp_metadata(source);
        assert!(adjusted.contains("exif:ColorSpace=\"1\""));
        assert!(adjusted.contains("photoshop:ICCProfile=\"sRGB\""));
        assert!(adjusted.contains("<exif:InteroperabilityIndex>R98</exif:InteroperabilityIndex>"));
        assert!(adjusted.contains("dc:creator=\"Camera A\""));
        assert!(!adjusted.contains("Adobe RGB (1998)"));
        assert!(matches!(srgb_xmp_metadata(&adjusted), std::borrow::Cow::Borrowed(_)));
        let unchanged = "free-form text about Adobe RGB (1998) is not a colour tag";
        assert!(matches!(srgb_xmp_metadata(unchanged), std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn metadata_container_keeps_exif_xmp_and_codestream_bytes() {
        let stream = vec![0xff, 0x0a, 0x12, 0x34];
        let exif = b"II*\0\x08\0\0\0";
        let xmp = "<xmp>test</xmp>";
        let result = jxl_with_metadata(stream.clone(), Some(exif), Some(xmp)).unwrap();
        assert_eq!(&result[..12], b"\0\0\0\x0cJXL \r\n\x87\n");
        assert_eq!(&result[12..32], b"\0\0\0\x14ftypjxl \0\0\0\0jxl ");
        let mut pos = 32;
        let mut exif_box = vec![0, 0, 0, 0];
        exif_box.extend_from_slice(exif);
        for (name, expected) in [(b"Exif".as_slice(), exif_box), (b"xml ".as_slice(), xmp.as_bytes().to_vec()), (b"jxlc".as_slice(), stream)] {
            let len = u32::from_be_bytes(result[pos..pos + 4].try_into().unwrap()) as usize;
            assert!(len >= 8);
            assert_eq!(&result[pos + 4..pos + 8], name);
            assert_eq!(&result[pos + 8..pos + len], expected.as_slice());
            pos += len;
        }
        assert_eq!(pos, result.len());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn missing_metadata_preserves_the_existing_bare_codestream() {
        let original = vec![0xff, 0x0a, 0x12, 0x34];
        assert_eq!(jxl_with_metadata(original.clone(), None, None).unwrap(), original);
        assert_eq!(jxl_with_metadata(original.clone(), Some(&[]), Some("")).unwrap(), original);
        assert!(jxl_with_metadata(vec![1, 2], Some(b"TIFF"), None).is_err());
    }
}
