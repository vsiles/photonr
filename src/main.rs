use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time;

use anyhow::Result;
use parry3d::query::{details::RayIntersection, Ray, RayCast};
// TODO: Compound is quite slow to use as a scene gatherer
use parry3d::shape::Ball;

mod math {
    use parry3d::na;

    pub type Scalar = f32;
    pub type Point = na::Point3<Scalar>;
    pub type Vector = na::Vector3<Scalar>;
    pub type Color = Vector;
    pub type Isometry = na::Isometry3<f32>;
}

use math::*;

const ASPECT_RATIO: Scalar = 16.0 / 9.0;
const IMAGE_WIDTH: usize = 400;
const VIEWPORT_HEIGHT: Scalar = 2.0;
const FOCAL_LENGTH: Scalar = 1.0;

const MAX_TOI: f32 = 1000.0;

type Col3 = Vector;

fn hit_shape(shape: &impl RayCast, iso: &Isometry, ray: &Ray) -> Option<RayIntersection> {
    // TODO: if the resulting toi is close to 0, normal might not be reliable
    shape.cast_ray_and_get_normal(iso, ray, MAX_TOI, true)
}

fn ray_color(r: &Ray) -> Col3 {
    let background_gradient = 0.5 * (r.dir.y + 1.0);
    let white = Color::new(1.0, 1.0, 1.0);
    let blue = Color::new(0.5, 0.7, 1.0);

    let small: (Isometry, _) = (Vector::new(0.0, 0.0, -1.0).into(), Ball::new(0.5));
    let big: (Isometry, _) = (Vector::new(0.0, -100.5, -1.0).into(), Ball::new(100.0));

    let shapes = vec![small, big];

    for (iso, shape) in &shapes {
        if let Some(intersection) = hit_shape(shape, iso, r) {
            let col: Col3 = intersection.normal + Vector::new(1.0, 1.0, 1.0);
            return col * 0.5;
        }
    }
    white.lerp(&blue, background_gradient)
}

pub fn write_color(v: &mut Vec<u8>, c: &Col3) {
    v.push((c.x * 255.999) as u8);
    v.push((c.y * 255.999) as u8);
    v.push((c.z * 255.999) as u8);
}

fn main() -> Result<()> {
    // very simple time computation
    let start = time::Instant::now();

    // Calculate the image height, and ensure that it's at least 1.
    let width: Scalar = IMAGE_WIDTH as Scalar;
    let mut height = width / ASPECT_RATIO;
    height = if height < 1.0 { 1.0 } else { height };
    let image_height: usize = height as usize;

    // Camera
    let viewport_width = VIEWPORT_HEIGHT * (width / height);
    let camera_center = Point::new(0.0, 0.0, 0.0);

    // Note: Right-handed System
    // meaning +x points right, +y points up and +z points towards the camera
    //
    // Calculate the vectors across the horizontal and down the vertical viewport edges.
    let viewport_u = Vector::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vector::new(0.0, -VIEWPORT_HEIGHT, 0.0);

    // Calculate the horizontal and vertical delta vectors from pixel to pixel.
    let pixel_delta_u = viewport_u / width;
    let pixel_delta_v = viewport_v / height;

    // Calculate the location of the upper left pixel.
    let viewport_upper_left: Point =
        (camera_center - Point::new(0.0, 0.0, FOCAL_LENGTH) - viewport_u / 2.0 - viewport_v / 2.0)
            .into();
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    // Render

    println!("Generating image: size {} x {}", IMAGE_WIDTH, image_height);
    let mut data: Vec<u8> = Vec::new();

    for j in 0..image_height {
        print!("\rScanlines remaining: {}", image_height - j);
        std::io::stdout().flush().unwrap();
        for i in 0..IMAGE_WIDTH {
            let pixel_center: Point =
                pixel00_loc + (i as Scalar * pixel_delta_u) + (j as Scalar * pixel_delta_v);
            let ray_direction: Vector = pixel_center - camera_center;
            let r = Ray::new(camera_center, ray_direction);
            let pixel_color = ray_color(&r);
            write_color(&mut data, &pixel_color);
        }
    }
    println!("\r                                ");
    std::io::stdout().flush().unwrap();

    let duration = start.elapsed();
    println!("\rDone in {} milliseconds", duration.as_millis());

    let path = Path::new(r"./image.png");
    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, IMAGE_WIDTH as u32, image_height as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;

    Ok(())
}
