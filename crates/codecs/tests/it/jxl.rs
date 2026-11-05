//! LightCraft → PhotoCraft JPEG XL integration, using procedural images only.
use photocraft_codecs::{Format, detect, from_extension};

#[test]
fn jxl_signatures_and_extensions() {
    assert_eq!(detect(b"\xff\x0a"), Some(Format::Jxl));
    assert_eq!(detect(b"\0\0\0\x0cJXL \r\n\x87\n"), Some(Format::Jxl));
    assert_eq!(from_extension("/pictures/Test.JXL"), Some(Format::Jxl));
    assert_eq!(Format::Jxl.mime_type(), "image/jxl");
}

#[cfg(all(feature = "jxl", not(target_arch = "wasm32")))]
mod native {
    use super::{Format, detect};
    use photocraft_codecs::{
        ChannelLayout, CodecError, DecodeOptions, EncodeOptions, FidelityWarning, Image, Limits, SampleType, decode, decode_with, encode,
        fidelity_warnings_with,
    };

    fn image(layout: ChannelLayout, depth: SampleType) -> Image {
        let ch = layout.channels();
        let values: Vec<u16> = (0..37 * 23 * ch).map(|i| ((i * 709 + i / ch * 13) % 65536) as u16).collect();
        if depth == SampleType::U16 {
            Image::from_u16(37, 23, layout, &values).unwrap()
        } else {
            Image::from_u8(37, 23, layout, values.iter().map(|s| (s >> 8) as u8).collect()).unwrap()
        }
    }

    #[test]
    fn jxl_lossless_rgb_rgba_gray_8_and_16_bit() {
        for layout in [ChannelLayout::Rgb, ChannelLayout::Rgba, ChannelLayout::Gray, ChannelLayout::GrayA] {
            for depth in [SampleType::U8, SampleType::U16] {
                let src = image(layout, depth);
                let data = encode(&src, Format::Jxl, &EncodeOptions::default()).unwrap();
                assert_eq!(detect(&data), Some(Format::Jxl));
                let back = decode(&data).unwrap();
                assert_eq!(back.dimensions(), src.dimensions());
                assert_eq!(back.sample_type(), depth);
                let expected = src.convert(back.layout(), depth);
                assert_eq!(back.data(), expected.data(), "{layout:?} {depth:?}");
            }
        }
    }

    #[test]
    fn jxl_lossy_transparency_and_quality_warning() {
        let src = image(ChannelLayout::Rgba, SampleType::U8);
        let mut opts = EncodeOptions::default();
        opts.jxl.lossless = false;
        opts.jxl.quality = 90;
        let bytes = encode(&src, Format::Jxl, &opts).unwrap();
        let back = decode(&bytes).unwrap();
        assert_eq!(back.layout(), ChannelLayout::Rgba);
        assert_eq!(back.dimensions(), src.dimensions());
        assert!(fidelity_warnings_with(&src, Format::Jxl, &opts).contains(&FidelityWarning::LossyCompression));
        for (a, b) in src.data().chunks_exact(4).zip(back.data().chunks_exact(4)) {
            assert_eq!(a[3], b[3], "lossy JPEG XL keeps alpha losslessly");
        }
    }

    #[test]
    fn jxl_xmp_and_metadata_disabled() {
        let mut src = image(ChannelLayout::Rgb, SampleType::U8);
        src.meta.exif = Some(crate::common::sample_exif());
        src.meta.xmp = Some("<x:xmpmeta xmlns:x=\"adobe:ns:meta/\">PhotoCraft JXL</x:xmpmeta>".into());
        let bytes = encode(&src, Format::Jxl, &EncodeOptions::default()).unwrap();
        assert_eq!(decode(&bytes).unwrap().meta.xmp, src.meta.xmp);
        assert_eq!(decode(&bytes).unwrap().meta.exif, src.meta.exif);
        let opts = EncodeOptions { embed_metadata: false, ..Default::default() };
        let bytes = encode(&src, Format::Jxl, &opts).unwrap();
        assert!(decode(&bytes).unwrap().meta.xmp.is_none());
        assert!(decode(&bytes).unwrap().meta.exif.is_none());
    }

    #[test]
    fn jxl_limits_and_malformed_data_are_errors() {
        let data = encode(&image(ChannelLayout::Rgb, SampleType::U16), Format::Jxl, &EncodeOptions::default()).unwrap();
        let opts = DecodeOptions { limits: Limits { max_width: 4, ..Default::default() }, ..Default::default() };
        assert!(matches!(decode_with(&data, &opts), Err(CodecError::LimitExceeded(_))));
        for bad in [b"\xff\x0a".as_slice(), &data[..data.len() / 2], b"\0\0\0\x0cJXL \r\n\x87\n"] {
            assert!(decode(bad).is_err());
        }
    }
}
