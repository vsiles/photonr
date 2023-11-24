use parry3d::query::Ray;
use rand::Rng;
use rayon::prelude::*;
use std::io::Write;

use crate::material::Material;
use crate::math::*;
use crate::world::*;

#[derive(Debug)]
pub struct Camera {
    aspect_ratio: f32,
    pub image_width: usize,
    pub image_height: usize,
    center: Point,
    pixel00_loc: Point,
    pixel_delta_u: Vector,
    pixel_delta_v: Vector,
    samples_per_pixel: usize,
    max_depth: usize,
}

// TODO: use Scalar everywhere

fn linear_to_gamma(linear_component: Scalar) -> Scalar {
    Scalar::sqrt(linear_component)
}

fn write_color(c: &Color, samples_per_pixel: usize) -> Vec<u8> {
    let factor: f32 = 1.0 / samples_per_pixel as f32;
    let c = c * factor;

    let r = linear_to_gamma(c.x);
    let g = linear_to_gamma(c.y);
    let b = linear_to_gamma(c.z);

    vec![
        (r * 255.999) as u8,
        (g * 255.999) as u8,
        (b * 255.999) as u8,
    ]
}

// - Make world background configurable
fn ray_color(rng: &mut rand::rngs::ThreadRng, ray: &Ray, world: &World, depth: usize) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let background_gradient = 0.5 * (ray.dir.y + 1.0);
    let white = Color::new(1.0, 1.0, 1.0);
    let blue = Color::new(0.5, 0.7, 1.0);

    if let Some((intersection, material)) = world.hit(ray) {
        if let Some((attenuation, scattered)) = material.scatter(rng, ray, &intersection) {
            let color = ray_color(rng, &scattered, world, depth - 1);
            return attenuation.component_mul(&color);
        } else {
            return Color::new(0.0, 0.0, 0.0);
        }
    }
    // No hit, let's have a nice background for now
    white.lerp(&blue, background_gradient)
}

impl Camera {
    pub fn dump_info(&self) {
        println!("image width: {}", self.image_width);
        println!("image height: {}", self.image_height);
        println!("aspect ratio: {}", self.aspect_ratio);
        println!("samples per pixel: {}", self.samples_per_pixel);
        println!("max depth: {}", self.max_depth);
    }

    fn pixel_sample_square(&self, rng: &mut rand::rngs::ThreadRng) -> Vector {
        // Returns a random point in the square surrounding a pixel at the origin.
        let offset: f32 = rng.gen_range(0.0..1.0);

        let px = -0.5 + offset;
        let py = -0.5 + offset;
        (px * self.pixel_delta_u) + (py * self.pixel_delta_v)
    }

    pub fn new(
        aspect_ratio: f32,
        image_width: usize,
        samples_per_pixel: usize,
        max_depth: usize,
    ) -> Camera {
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
            aspect_ratio,
            image_width,
            image_height,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            samples_per_pixel,
            max_depth,
        }
    }

    fn get_ray(&self, rng: &mut rand::rngs::ThreadRng, i: usize, j: usize) -> Ray {
        let pixel_center: Point = self.pixel00_loc
            + (i as Scalar * self.pixel_delta_u)
            + (j as Scalar * self.pixel_delta_v);

        let pixel_sample: Point = pixel_center + self.pixel_sample_square(rng);

        let ray_direction: Vector = pixel_sample - self.center;
        Ray::new(self.center, ray_direction)
    }

    pub fn render(&self, world: World) -> Vec<u8> {
        println!(
            "Generating image: size {} x {}",
            self.image_width, self.image_height
        );

        // very simple time computation
        let start = std::time::Instant::now();

        let cnt = std::sync::Arc::new(std::sync::Mutex::new(0));
        let data = (0..self.image_height)
            .into_par_iter()
            .map(|j| {
                {
                    let lock = cnt.lock();
                    let val: u32 = *lock.unwrap();
                    let ratio = val * 100 / self.image_height as u32;
                    print!("\rCurrent progress: {} %", ratio);
                }
                let mut rng = rand::thread_rng();

                std::io::stdout().flush().unwrap();
                let res = (0..self.image_width)
                    .map(|i| {
                        let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                        for _sample in 0..self.samples_per_pixel {
                            let ray = self.get_ray(&mut rng, i, j);
                            pixel_color += ray_color(&mut rng, &ray, &world, self.max_depth);
                        }
                        write_color(&pixel_color, self.samples_per_pixel)
                    })
                    .collect::<Vec<_>>();
                let mut lock = cnt.lock().unwrap();
                *lock += 1;
                res
            })
            .collect::<Vec<_>>();
        println!("\r                                          ");
        std::io::stdout().flush().unwrap();

        let duration = start.elapsed();
        println!("\rDone in {} milliseconds", duration.as_millis());
        let data: Vec<_> = data.into_iter().flatten().collect();
        data.into_iter().flatten().collect()
    }
}
