use parry3d::query::{Ray, RayIntersection};

use serde::{Deserialize, Serialize};

use crate::math::*;

pub trait Material {
    fn scatter(
        &self,
        rng: &mut rand::rngs::ThreadRng,
        ray_in: &Ray,
        intersection: &RayIntersection,
    ) -> Option<(Color, Ray)>;
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Lambertian {
    albedo: Color,
}

// impl Lambertian {
//     pub fn new(albedo: Color) -> Lambertian {
//         Lambertian { albedo }
//     }
// }

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

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Metal {
    albedo: Color,
    fuzz: f32,
}

// impl Metal {
//     pub fn new(albedo: Color, fuzz: f32) -> Metal {
//         Metal {
//             albedo,
//             fuzz: fuzz.clamp(0.0, 1.0),
//         }
//     }
// }

impl Material for Metal {
    fn scatter(
        &self,
        rng: &mut rand::rngs::ThreadRng,
        ray_in: &Ray,
        intersection: &RayIntersection,
    ) -> Option<(Color, Ray)> {
        // TODO: do the math, should I normalize ray_in.dir ?
        let reflected = vector_reflect(&ray_in.dir, &intersection.normal);
        let hit = ray_in.point_at(intersection.toi);
        let scattered = Ray::new(hit, reflected + self.fuzz * random_unit_vector(rng));
        if scattered.dir.dot(&intersection.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub enum MaterialKind {
    Lambertian(Lambertian),
    Metal(Metal),
}

impl Material for MaterialKind {
    fn scatter(
        &self,
        rng: &mut rand::rngs::ThreadRng,
        ray_in: &Ray,
        intersection: &RayIntersection,
    ) -> Option<(Color, Ray)> {
        match self {
            MaterialKind::Lambertian(mat) => mat.scatter(rng, ray_in, intersection),
            MaterialKind::Metal(mat) => mat.scatter(rng, ray_in, intersection),
        }
    }
}
