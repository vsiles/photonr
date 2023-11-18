use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::Parser;
use encoding_rs::Encoding;

mod cli;
mod json;
mod material;
mod math;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const IMAGE_WIDTH: usize = 400;

mod camera;
use camera::Camera;

mod world;
use world::*;

/// Helper function to deal with windows (utf16) vs other systems (utf8)

fn detect_encoding(bytes: &[u8]) -> String {
    let (encoding, _) = Encoding::for_bom(bytes).unwrap(); // TODO: deal with errors
    eprintln!("Tentative encoding: {}", encoding.name());
    let (content, actual_encoding, malformed) = encoding.decode(bytes);
    eprintln!("Actual encoding: {}", actual_encoding.name());
    eprintln!("malformed sequences spotted ? {}", malformed);
    content.to_string()
}

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let width = cli.width.unwrap_or(IMAGE_WIDTH);
    let aspect_ratio = cli.aspect_ratio.unwrap_or(ASPECT_RATIO);
    let samples_per_pixel = cli.samples_per_pixel.unwrap_or(10);
    let max_depth = cli.max_depth.unwrap_or(10);

    let camera = Camera::new(aspect_ratio, width, samples_per_pixel, max_depth);

    if cli.dump_info {
        camera.dump_info()
    }

    let mut scene = File::open(r"./scene.json").context("Can't open default scene 'scene.json'")?;

    // Read the file as bytes
    let mut buffer = Vec::new();
    scene
        .read_to_end(&mut buffer)
        .context("Failed to read scene as bytes")?;

    // Detect the encoding and load the content as a string
    let scene_description = detect_encoding(&buffer);

    let jworld: json::World =
        serde_json::from_str(&scene_description).context("Failed to read json input")?;
    let world: World = jworld.into();

    // Render
    let data = camera.render(world);

    // Save as PNG
    let img = match image::RgbImage::from_vec(
        camera.image_width as u32,
        camera.image_height as u32,
        data,
    ) {
        Some(img) => img,
        None => bail!("Failed to create RGB image"),
    };

    let img = image::DynamicImage::ImageRgb8(img);
    let path = Path::new(r"./image.png");
    img.save(path).context("Failed to save PNG image")?;

    Ok(())
}
