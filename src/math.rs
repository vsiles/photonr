use parry3d::na;

pub type Scalar = f32;
pub type Point = na::Point3<Scalar>;
pub type Vector = na::Vector3<Scalar>;
pub type Color = Vector;
pub type Isometry = na::Isometry3<f32>;

use rand::Rng;

#[allow(dead_code)]
pub fn random_vector(rng: &mut rand::rngs::ThreadRng) -> Vector {
    Vector::new(rng.gen(), rng.gen(), rng.gen())
}

fn random_vector_rang(rng: &mut rand::rngs::ThreadRng, min: Scalar, max: Scalar) -> Vector {
    let x: f32 = rng.gen_range(min..max);
    let y: f32 = rng.gen_range(min..max);
    let z: f32 = rng.gen_range(min..max);
    Vector::new(x, y, z)
}

fn random_vector_in_unit_sphere(rng: &mut rand::rngs::ThreadRng) -> Vector {
    loop {
        let v = random_vector_rang(rng, -1.0, 1.0);
        if v.norm_squared() < 1.0 {
            break v;
        }
    }
}

pub fn random_unit_vector(rng: &mut rand::rngs::ThreadRng) -> Vector {
    random_vector_in_unit_sphere(rng).normalize()
}

#[allow(dead_code)]
pub fn random_unit_vector_on_hemisphere(
    rng: &mut rand::rngs::ThreadRng,
    normal: &Vector,
) -> Vector {
    let unit = random_unit_vector(rng);
    if unit.dot(normal) > 0.0 {
        unit
    } else {
        -unit
    }
}
