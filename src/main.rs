use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::Result;
use parry3d::shape::Ball;

mod math;
use math::*;

const ASPECT_RATIO: Scalar = 16.0 / 9.0;
const IMAGE_WIDTH: usize = 400;

mod camera;
use camera::Camera;

fn main() -> Result<()> {
    let camera = Camera::new(ASPECT_RATIO, IMAGE_WIDTH);

    let small: (Isometry, _) = (Vector::new(0.0, 0.0, -1.0).into(), Ball::new(0.5));
    let big: (Isometry, _) = (Vector::new(0.0, -100.5, -1.0).into(), Ball::new(100.0));

    let world = vec![small, big];

    // Render
    let data = camera.render(&world);

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
