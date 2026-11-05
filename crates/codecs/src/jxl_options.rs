//! JPEG XL settings adapted from LightCraft's `jxl_encode` (MIT OR Apache-2.0).

/// Encoder effort. Only the speeds LightCraft can reliably read back are offered.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum JxlEffort {
    Fast,
    #[default]
    Normal,
}

/// JPEG XL export settings, shared by the GUI, CLI and codec API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JxlOptions {
    /// Quality 1–100; ignored for mathematically lossless compression.
    pub quality: u8,
    pub lossless: bool,
    pub effort: JxlEffort,
    pub progressive: bool,
    /// Samples are 16-bit Rec.2020 PQ (SDR reference white: 203 cd/m²).
    pub hdr: bool,
}

impl Default for JxlOptions {
    fn default() -> Self {
        Self { quality: 90, lossless: true, effort: JxlEffort::Normal, progressive: false, hdr: false }
    }
}
