//! Generate a small, synthetic 18-layer PSD for the Linux memory-admission regression.
//! No photographs, fonts, third-party images or private source files are used.

use photocraft_psd::{Compression, LayerSpec, PixelData, PsdBuilder, PsdFile};
use std::io::Write;

const WIDTH: usize = 1491;
const HEIGHT: usize = 1055;
const LAYERS: usize = 18;

fn color(index: usize) -> [u8; 4] {
    match index % 6 {
        0 => [36, 124, 210, 255],
        1 => [225, 91, 72, 255],
        2 => [58, 166, 111, 255],
        3 => [236, 181, 48, 255],
        4 => [143, 100, 191, 255],
        _ => [38, 170, 181, 255],
    }
}

fn paint_rectangle(pixels: &mut [u8], index: usize) -> Result<(), Box<dyn std::error::Error>> {
    let left = 50 + index % 6 * 230;
    let top = 70 + index / 6 * 310;
    let rgba = color(index);
    for y in top..top + 240 {
        for x in left..left + 190 {
            let at = (y * WIDTH + x) * 4;
            pixels.get_mut(at..at + 4).ok_or("rectangle exceeds the fixed canvas")?.copy_from_slice(&rgba);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os().nth(1).ok_or("pass a new output PSD path")?;
    let width = u32::try_from(WIDTH)?;
    let height = u32::try_from(HEIGHT)?;
    let mut builder = PsdBuilder::new(width, height).compression(Compression::Zip);
    let mut composite = [242, 244, 248, 255].repeat(WIDTH * HEIGHT);
    builder.push_layer(LayerSpec::new("Background", 0, 0, width, height, PixelData::Rgba8(composite.clone())));
    for index in 0..LAYERS - 1 {
        let mut pixels = vec![0; WIDTH * HEIGHT * 4];
        paint_rectangle(&mut pixels, index)?;
        paint_rectangle(&mut composite, index)?;
        builder.push_layer(LayerSpec::new(format!("Rectangle {:02}", index + 1), 0, 0, width, height, PixelData::Rgba8(pixels)));
    }
    builder.composite(PixelData::Rgba8(composite));
    let bytes = builder.to_bytes()?;
    let reopened = PsdFile::from_bytes(&bytes)?;
    if reopened.layers().len() != LAYERS {
        return Err("generated PSD has an unexpected layer count".into());
    }
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&output)?;
    file.write_all(&bytes)?;
    println!("{}: {} bytes, {width} x {height}, {LAYERS} layers", std::path::Path::new(&output).display(), bytes.len());
    Ok(())
}
