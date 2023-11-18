use std::sync::Arc;

use parry3d::query::{Ray, RayCast, RayIntersection};
use parry3d::shape::Ball;

use crate::material::*;
use crate::math::*;

use crate::json;

pub trait Entity {
    fn hit(&self, ray: &Ray) -> Option<RayIntersection>;

    fn material(&self) -> &MaterialKind;
}

// TODO:
// - sphere is currently solid only
// - MAX_TOI is not configurable
const MAX_TOI: f32 = 1000.0;

pub struct Sphere {
    ball: Ball,
    isometry: Isometry,
    material: MaterialKind,
}

impl Sphere {
    pub fn new(center: Point, radius: Scalar, material: MaterialKind) -> Self {
        let ball = Ball::new(radius);
        let isometry: Isometry = center.into();
        Sphere {
            ball,
            isometry,
            material,
        }
    }
}

impl Entity for Sphere {
    fn material(&self) -> &MaterialKind {
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

    pub fn hit(&self, ray: &Ray) -> Option<(RayIntersection, &MaterialKind)> {
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

fn vec_to_point(vec: Vec<f32>) -> Point {
    // TODO: check size and report error
    let x = vec[0];
    let y = vec[1];
    let z = vec[2];
    Point::new(x, y, z)
}

impl From<json::World> for World {
    fn from(value: json::World) -> Self {
        let json::World { materials, shapes } = value;
        let mut world = World::new();
        for shape in shapes {
            match shape {
                json::Shape::Sphere(json::Sphere {
                    center,
                    radius,
                    material,
                }) => {
                    let mat = materials.get(&material).unwrap().clone();
                    let sphere = Sphere::new(vec_to_point(center), radius, mat);
                    world.add(Arc::new(sphere));
                }
            }
        }
        world
    }
}
