use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;

mod math;
use math::*;

mod material;
use material::*;

const ASPECT_RATIO: Scalar = 16.0 / 9.0;
const IMAGE_WIDTH: usize = 400;

mod camera;
use camera::Camera;

mod world;
use world::*;

fn main() -> Result<()> {
    let camera = Camera::new(ASPECT_RATIO, IMAGE_WIDTH);

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
    let path = Path::new(r"./image.png");
    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, camera.image_width as u32, camera.image_height as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;

    Ok(())
}
