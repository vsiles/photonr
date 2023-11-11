use std::sync::Arc;

use parry3d::query::{Ray, RayCast, RayIntersection};
use parry3d::shape::Ball;

use crate::material::*;
use crate::math::*;

pub trait Entity {
    fn hit(&self, ray: &Ray) -> Option<RayIntersection>;

    fn material(&self) -> &dyn Material;
}

// TODO:
// - sphere is currently solid only
// - MAX_TOI is not configurable
const MAX_TOI: f32 = 1000.0;

pub struct Sphere<M> {
    ball: Ball,
    isometry: Isometry,
    material: M,
}

impl<M> Sphere<M> {
    pub fn new(center: Point, radius: Scalar, material: M) -> Self {
        let ball = Ball::new(radius);
        let isometry: Isometry = center.into();
        Sphere {
            ball,
            isometry,
            material,
        }
    }
}

unsafe impl<M> Sync for Sphere<M> where M: Sync {}
unsafe impl<M> Send for Sphere<M> where M: Send {}

impl<M> Entity for Sphere<M>
where
    M: Material,
{
    fn material(&self) -> &dyn Material {
        &self.material
    }

    fn hit(&self, ray: &Ray) -> Option<RayIntersection> {
        let result = self
            .ball
            .cast_ray_and_get_normal(&self.isometry, ray, MAX_TOI, true);
        result.filter(|intersection| intersection.toi.abs() > f32::EPSILON)
    }
}

pub struct World {
    entities: Vec<Arc<dyn Entity + Sync + Send>>,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
        }
    }

    pub fn add<E>(&mut self, e: Arc<E>)
    where
        E: Entity + Send + Sync + 'static,
    {
        self.entities.push(e)
    }

    pub fn hit(&self, ray: &Ray) -> Option<(RayIntersection, &dyn Material)> {
        let mut ret = None;
        let mut closest_toi = MAX_TOI;

        for entity in &self.entities {
            if let Some(intersection) = entity.hit(ray) {
                if intersection.toi < closest_toi {
                    closest_toi = intersection.toi;
                    ret = Some((intersection, entity.material()));
                }
            }
        }
        ret
    }
}
