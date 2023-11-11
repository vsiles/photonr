use parry3d::query::{Ray, RayIntersection};

use crate::math::*;

pub trait Material {
    fn scatter(
        &self,
        rng: &mut rand::rngs::ThreadRng,
        ray_in: &Ray,
        intersection: &RayIntersection,
    ) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    albedo: Color,
}

unsafe impl Sync for Lambertian {}
unsafe impl Send for Lambertian {}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        rng: &mut rand::rngs::ThreadRng,
        ray_in: &Ray,
        intersection: &RayIntersection,
    ) -> Option<(Color, Ray)> {
        let mut scatter_direction = intersection.normal + random_unit_vector(rng);
        if vector_near_zero(&scatter_direction) {
            scatter_direction = intersection.normal;
        }
        let scattered = Ray::new(ray_in.point_at(intersection.toi), scatter_direction);
        Some((self.albedo, scattered))
    }
}

pub struct Metal {
    albedo: Color,
}

unsafe impl Sync for Metal {}
unsafe impl Send for Metal {}

impl Metal {
    pub fn new(albedo: Color) -> Metal {
        Metal { albedo }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        _rng: &mut rand::rngs::ThreadRng,
        ray_in: &Ray,
        intersection: &RayIntersection,
    ) -> Option<(Color, Ray)> {
        let reflected = vector_reflect(&ray_in.dir, &intersection.normal);
        Some((
            self.albedo,
            Ray::new(ray_in.point_at(intersection.toi), reflected),
        ))
    }
}
