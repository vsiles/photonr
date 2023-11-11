// TODO: Compound is quite slow to use as a scene gatherer
use parry3d::query::{Ray, RayCast};
use rand::Rng;

use crate::math::*;
use rayon::prelude::*;
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

fn ray_color<R>(
    rng: &mut rand::rngs::ThreadRng,
    ray: &Ray,
    world: &Vec<(Isometry, R)>,
    depth: usize,
) -> Color
where
    R: RayCast,
{
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let background_gradient = 0.5 * (ray.dir.y + 1.0);
    let white = Color::new(1.0, 1.0, 1.0);
    let blue = Color::new(0.5, 0.7, 1.0);

    for (iso, shape) in world {
        // TODO: if the resulting toi is close to 0, normal might not be reliable
        if let Some(intersection) = shape.cast_ray_and_get_normal(iso, ray, MAX_TOI, true) {
            if intersection.toi.abs() < f32::EPSILON {
                continue;
            }
            let loc = ray.point_at(intersection.toi);
            // random diffusion direction
            // let direction = random_unit_vector_on_hemisphere(rng, &intersection.normal);
            // Lambertian model
            let direction = intersection.normal + random_unit_vector(rng);
            let secondary_ray = Ray::new(loc, direction);
            let diffuse = ray_color(rng, &secondary_ray, world, depth - 1);
            return 0.5 * diffuse;
        }
    }
    white.lerp(&blue, background_gradient)
}
impl Camera {
    fn pixel_sample_square(&self, rng: &mut rand::rngs::ThreadRng) -> Vector {
        // Returns a random point in the square surrounding a pixel at the origin.
        let offset: f32 = rng.gen_range(0.0..1.0);

        let px = -0.5 + offset;
        let py = -0.5 + offset;
        (px * self.pixel_delta_u) + (py * self.pixel_delta_v)
    }

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
            samples_per_pixel: 10,
            max_depth: 10,
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

    // TODO replace Vec with Iterator
    pub fn render<R>(&self, world: &Vec<(Isometry, R)>) -> Vec<u8>
    where
        R: RayCast + std::marker::Sync,
    {
        println!(
            "Generating image: size {} x {}",
            self.image_width, self.image_height
        );

        // very simple time computation
        let start = std::time::Instant::now();

        let data = (0..self.image_height)
            .into_par_iter()
            .map(|j| {
                print!("\rScanlines remaining: {}", self.image_height - j);
                let mut rng = rand::thread_rng();

                std::io::stdout().flush().unwrap();
                (0..self.image_width)
                    .map(|i| {
                        let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                        for _sample in 0..self.samples_per_pixel {
                            let r = self.get_ray(&mut rng, i, j);
                            pixel_color += ray_color(&mut rng, &r, world, self.max_depth);
                        }
                        write_color(&pixel_color, self.samples_per_pixel)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        println!("\r                                ");
        std::io::stdout().flush().unwrap();

        let duration = start.elapsed();
        println!("\rDone in {} milliseconds", duration.as_millis());
        let data: Vec<_> = data.into_iter().flatten().collect();
        data.into_iter().flatten().collect()
    }
}
