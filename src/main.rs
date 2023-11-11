use std::path::Path;
use std::sync::Arc;

use clap::Parser;

use anyhow::{bail, Context, Result};

mod cli;

mod math;
use math::*;

mod material;
use material::*;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const IMAGE_WIDTH: usize = 400;

mod camera;
use camera::Camera;

mod world;
use world::*;

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

    let mut world = World::new();

    let material_ground = Lambertian::new(Color::new(0.8, 0.8, 0.0));
    let material_center = Lambertian::new(Color::new(0.7, 0.3, 0.3));
    let material_left = Metal::new(Color::new(0.8, 0.8, 0.8));
    let material_right = Metal::new(Color::new(0.8, 0.6, 0.2));

    world.add(Arc::new(Sphere::new(
        Point::new(0.0, -100.5, -1.0),
        100.0,
        material_ground,
    )));
    world.add(Arc::new(Sphere::new(
        Point::new(0.0, 0.0, -1.0),
        0.5,
        material_center,
    )));
    world.add(Arc::new(Sphere::new(
        Point::new(-1.0, 0.0, -1.0),
        0.5,
        material_left,
    )));
    world.add(Arc::new(Sphere::new(
        Point::new(1.0, 0.0, -1.0),
        0.5,
        material_right,
    )));

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
