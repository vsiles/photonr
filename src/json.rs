use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::material;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct Sphere {
    pub center: Vec<f32>,
    pub radius: f32,
    pub material: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    Sphere(Sphere),
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct World {
    pub materials: HashMap<String, material::MaterialKind>,
    pub shapes: Vec<Shape>,
}
