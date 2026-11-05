//! LightCraft's JPEG XL mechanism, adapted to PhotoCraft's samples and limits.
//! Original LightCraft code: MIT OR Apache-2.0, see `NOTICE-JXL.md`.
//! No external codec executable or shared JPEG XL library is required.
//! The low-level lossy encoder expects sRGB samples; the document IO layer performs
//! profile conversion before calling it. Lossless mode preserves an embedded RGB ICC.

use crate::fidelity::Plan;
use crate::{CodecError, EncodeOptions, Format, Image, Limits};

const F: Format = Format::Jxl;

pub(crate) fn decode(bytes: &[u8], limits: &Limits) -> Result<Image, CodecError> {
    #[cfg(feature = "jxl")]
    {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| decoder::decode(bytes, limits)))
            .unwrap_or_else(|_| Err(CodecError::malformed(F, "JPEG XL decoder panicked")))
    }
    #[cfg(not(feature = "jxl"))]
    {
        let _ = (bytes, limits);
        Err(CodecError::unsupported(F, "JPEG XL decoding is not enabled"))
    }
}

pub(crate) fn encode(src: &Image, plan: Plan, opts: &EncodeOptions) -> Result<Vec<u8>, CodecError> {
    #[cfg(all(feature = "jxl", not(target_arch = "wasm32")))]
    {
        // Like LightCraft, turn a third-party encoder assertion into an export error.
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| encoder::encode(src, plan, opts)))
            .unwrap_or_else(|_| Err(CodecError::encode(F, "JPEG XL encoder panicked")))
    }
    #[cfg(any(not(feature = "jxl"), target_arch = "wasm32"))]
    {
        if !opts.jxl.lossless || opts.jxl.hdr || opts.jxl.progressive || opts.jxl.effort != crate::JxlEffort::Normal {
            return Err(CodecError::unsupported(F, "these JPEG XL settings require the desktop build with the jxl feature"));
        }
        super::jxl_basic::encode(src, plan)
    }
}

#[cfg(feature = "jxl")]
mod decoder {
    use super::{CodecError, F, Image, Limits};
    use crate::options::alloc_zeroed;
    use crate::{ChannelLayout, DecodeWarning, Metadata, SampleType};
    use jxl_oxide::{AllocTracker, AuxBoxData, InitializeResult, JxlImage, PixelFormat};

    const INPUT_CHUNK: usize = 4096;

    fn err(e: impl std::fmt::Display) -> CodecError {
        CodecError::malformed(F, e)
    }

    fn layout(img: &JxlImage) -> ChannelLayout {
        match img.pixel_format() {
            PixelFormat::Gray => ChannelLayout::Gray,
            PixelFormat::Graya => ChannelLayout::GrayA,
            PixelFormat::Rgb => ChannelLayout::Rgb,
            PixelFormat::Rgba => ChannelLayout::Rgba,
            PixelFormat::Cmyk => ChannelLayout::Cmyk,
            PixelFormat::Cmyka => ChannelLayout::CmykA,
        }
    }

    fn sample(img: &JxlImage) -> SampleType {
        let meta = &img.image_header().metadata;
        let depths = std::iter::once(meta.bit_depth).chain(meta.ec_info.iter().filter(|e| e.is_alpha()).map(|e| e.bit_depth));
        let mut sample = SampleType::U8;
        for depth in depths {
            if matches!(depth, jxl_oxide::image::BitDepth::FloatSample { .. }) || depth.bits_per_sample() > 16 {
                return SampleType::F32;
            }
            if depth.bits_per_sample() > 8 {
                sample = SampleType::U16;
            }
        }
        sample
    }

    // Bounded feeds let us reject excessive dimensions before loading image frames.
    // Feeding a complete file at once would also copy an arbitrary container tail.
    fn open(bytes: &[u8], limits: &Limits) -> Result<JxlImage, CodecError> {
        let tracker = AllocTracker::with_limit(limits.alloc_usize());
        let mut uninit = JxlImage::builder().alloc_tracker(tracker).build_uninit();
        let mut rest = bytes;
        let mut window = INPUT_CHUNK;
        let mut img = loop {
            let consumed = uninit.feed_bytes(rest.get(..window).unwrap_or(rest)).map_err(err)?;
            rest = rest.get(consumed..).unwrap_or_default();
            match uninit.try_init().map_err(err)? {
                InitializeResult::Initialized(img) => break img,
                InitializeResult::NeedMoreData(more) => uninit = more,
            }
            if consumed == 0 {
                if window >= rest.len() {
                    return Err(err("truncated JPEG XL header"));
                }
                window = window.saturating_mul(2);
            }
        };
        limits.check(img.width(), img.height(), layout(&img), sample(&img))?;
        // Rendering also needs floating-point working planes for colour and extra channels.
        limits.check_bytes(img.width(), img.height(), 4 * (3 + img.image_header().metadata.ec_info.len() as u64))?;
        // A JPEG XL container may place EXIF / XMP boxes *after* the last image
        // frame. is_loading_done() only describes the codestream, not the rest
        // of the container: stopping there silently discards trailing metadata.
        // Continue feeding the whole file in bounded pieces before finalize().
        let mut window = INPUT_CHUNK;
        while !rest.is_empty() {
            let consumed = img.feed_bytes(rest.get(..window).unwrap_or(rest)).map_err(err)?;
            if consumed == 0 {
                // Like the header parser, retry with a larger slice if a box
                // header requires more contiguous input to make progress.
                if window >= rest.len() {
                    return Err(err("truncated JPEG XL container"));
                }
                window = window.saturating_mul(2);
                continue;
            }
            rest = rest.get(consumed..).unwrap_or_default();
            window = INPUT_CHUNK;
        }
        img.finalize().map_err(err)?;
        if !img.is_loading_done() {
            return Err(err("truncated JPEG XL image"));
        }
        Ok(img)
    }

    pub(super) fn decode(bytes: &[u8], limits: &Limits) -> Result<Image, CodecError> {
        let img = open(bytes, limits)?;
        let (w, h, layout, sample) = (img.width(), img.height(), layout(&img), sample(&img));
        let render = img.render_frame(0).map_err(err)?;
        let mut stream = render.stream();
        if stream.width() != w || stream.height() != h || stream.channels() as usize != layout.channels() {
            return Err(err("inconsistent JPEG XL frame dimensions or channels"));
        }
        let count = (w as usize).checked_mul(h as usize).and_then(|n| n.checked_mul(layout.channels())).ok_or_else(|| err("JPEG XL image too large"))?;
        let mut out = match sample {
            SampleType::U8 => {
                let mut data = alloc_zeroed::<u8>(count)?;
                if stream.write_to_buffer(&mut data) != count {
                    return Err(err("incomplete JPEG XL frame"));
                }
                Image::from_u8(w, h, layout, data)?
            }
            SampleType::U16 => {
                let mut data = alloc_zeroed::<u16>(count)?;
                if stream.write_to_buffer(&mut data) != count {
                    return Err(err("incomplete JPEG XL frame"));
                }
                Image::from_u16(w, h, layout, &data)?
            }
            _ => {
                let mut data = alloc_zeroed::<f32>(count)?;
                if stream.write_to_buffer(&mut data) != count {
                    return Err(err("incomplete JPEG XL frame"));
                }
                Image::from_f32(w, h, layout, &data)?
            }
        };
        // Keep the profile of the actual rendered samples, including enum-tagged P3/HDR files.
        out.icc = Some(img.original_icc().map_or_else(|| img.rendered_icc(), <[u8]>::to_vec));
        let exif = match img.aux_boxes().first_exif().map_err(err)? {
            AuxBoxData::Data(exif) => Some(exif.payload().to_vec()),
            _ => None,
        };
        let xmp = match img.aux_boxes().first_xml() {
            AuxBoxData::Data(xmp) => Some(String::from_utf8_lossy(xmp).trim_end_matches('\0').to_string()),
            _ => None,
        };
        // jxl-oxide applies codestream orientation. Normalize EXIF/XMP so it isn't applied twice.
        out.meta =
            Metadata { exif: exif.map(|e| crate::upright_exif(&e).into_owned()), xmp: xmp.map(|x| crate::upright_xmp(&x).into_owned()), ..Default::default() };
        if img.num_loaded_keyframes() > 1 {
            out.warnings.push(DecodeWarning::MoreFrames { total: u32::try_from(img.num_loaded_keyframes()).ok() });
        }
        Ok(out)
    }
}

#[cfg(all(feature = "jxl", not(target_arch = "wasm32")))]
mod encoder {
    use super::{CodecError, EncodeOptions, F, Image, Plan};
    use crate::{ChannelLayout, JxlEffort, SampleType};
    use jixel::{ColorEncoding, EncodeConfig, Speed};

    pub(super) fn encode(src: &Image, plan: Plan, opts: &EncodeOptions) -> Result<Vec<u8>, CodecError> {
        let img = src.converted(plan.layout, plan.sample);
        let mut config = EncodeConfig::default()
            .with_quality(f32::from(opts.jxl.quality.clamp(1, 100)))
            .with_color_encoding(if opts.jxl.hdr { ColorEncoding::bt2020_pq() } else { ColorEncoding::srgb() })
            .with_lossless(opts.jxl.lossless)
            .with_progressive(opts.jxl.progressive)
            .with_speed(match opts.jxl.effort {
                JxlEffort::Fast => Speed::Fastest,
                JxlEffort::Normal => Speed::Fast,
            });
        if opts.jxl.hdr {
            config = config.with_intensity_target(10000.0);
        }
        if opts.jxl.lossless
            && opts.embed_icc
            && let Some(icc) = &img.icc
        {
            config = config.with_icc_profile(icc.clone());
        }
        if opts.embed_metadata {
            if let Some(exif) = super::super::webp::export_exif(&img) {
                config = config.with_exif(exif);
            }
            if let Some(xmp) = &img.meta.xmp {
                config = config.with_xmp(crate::resolution::export_xmp(xmp, img.meta.dpi).as_bytes().to_vec());
            }
        }
        let (w, h) = (img.width() as usize, img.height() as usize);
        let alpha = img.layout() == ChannelLayout::Rgba;
        let encoded = match img.sample_type() {
            SampleType::U8 => {
                if alpha {
                    jixel::encode_image_with_alpha(img.data(), w, h, &config)
                } else {
                    jixel::encode_image(img.data(), w, h, &config)
                }
            }
            SampleType::U16 => {
                let data = img.to_u16_samples().ok_or_else(|| CodecError::encode(F, "invalid 16-bit samples"))?;
                if alpha { jixel::encode_image_with_alpha_16bit(&data, w, h, &config) } else { jixel::encode_image_16bit(&data, w, h, &config) }
            }
            _ => return Err(CodecError::encode(F, "JPEG XL export requires 8- or 16-bit samples")),
        };
        encoded.map_err(|e| CodecError::encode(F, e))
    }
}
