// TODO: Compound is quite slow to use as a scene gatherer
use parry3d::query::{Ray, RayCast};

use crate::math::*;
use std::io::Write;

const MAX_TOI: f32 = 1000.0;

pub struct Camera {
    // aspect_ratio: f32,
    pub image_width: usize,
    pub image_height: usize,
    center: Point,
    pixel00_loc: Point,
    pixel_delta_u: Vector,
    pixel_delta_v: Vector,
}

fn write_color(v: &mut Vec<u8>, c: &Color) {
    v.push((c.x * 255.999) as u8);
    v.push((c.y * 255.999) as u8);
    v.push((c.z * 255.999) as u8);
}

impl Camera {
    pub fn new(aspect_ratio: f32, image_width: usize) -> Camera {
        let width = image_width as f32;
        let mut height = width / aspect_ratio;
        height = if height < 1.0 { 1.0 } else { height };
        let image_height = height as usize;

        let center = Point::new(0.0, 0.0, 0.0);

        // Determine viewport dimensions.
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (width / height);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = Vector::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vector::new(0.0, -viewport_height, 0.0);

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / width;
        let pixel_delta_v = viewport_v / height;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left: Point =
            (center - Point::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0)
                .into();
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Camera {
            // aspect_ratio,
            image_width,
            image_height,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
        }
    }

    // TODO replace Vec with Iterator
    pub fn render<R>(&self, world: &Vec<(Isometry, R)>) -> Vec<u8>
    where
        R: RayCast,
    {
        println!(
            "Generating image: size {} x {}",
            self.image_width, self.image_height
        );
        let mut data: Vec<u8> = Vec::new();

        // very simple time computation
        let start = std::time::Instant::now();

        for j in 0..self.image_height {
            print!("\rScanlines remaining: {}", self.image_height - j);
            std::io::stdout().flush().unwrap();
            for i in 0..self.image_width {
                let pixel_center: Point = self.pixel00_loc
                    + (i as Scalar * self.pixel_delta_u)
                    + (j as Scalar * self.pixel_delta_v);
                let ray_direction: Vector = pixel_center - self.center;
                let r = Ray::new(self.center, ray_direction);
                let pixel_color = Self::ray_color(&r, world);
                write_color(&mut data, &pixel_color);
            }
        }
        println!("\r                                ");
        std::io::stdout().flush().unwrap();

        let duration = start.elapsed();
        println!("\rDone in {} milliseconds", duration.as_millis());
        data
    }

    fn ray_color<R>(ray: &Ray, world: &Vec<(Isometry, R)>) -> Color
    where
        R: RayCast,
    {
        let background_gradient = 0.5 * (ray.dir.y + 1.0);
        let white = Color::new(1.0, 1.0, 1.0);
        let blue = Color::new(0.5, 0.7, 1.0);

        for (iso, shape) in world {
            // TODO: if the resulting toi is close to 0, normal might not be reliable
            if let Some(intersection) = shape.cast_ray_and_get_normal(iso, ray, MAX_TOI, true) {
                let col: Color = intersection.normal + Vector::new(1.0, 1.0, 1.0);
                return col * 0.5;
            }
        }
        white.lerp(&blue, background_gradient)
    }
}
