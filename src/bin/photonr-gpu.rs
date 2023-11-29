use rand::Rng;
use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::Parser;
// WARNING
// struct are tricky with OclPrm
// https://github.com/cogciprocate/ocl/pull/168/files
use ocl::ProQue;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const IMAGE_WIDTH: usize = 400;

use photonr::cli;
use photonr::json;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, PartialEq)]
struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    // opencl has float2 and float4, but no float3
    // We'll not really use it for now
    w: f32,
}
unsafe impl ocl::OclPrm for Vec4 {}

impl Vec4 {
    fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
        Vec4 { x, y, z, w }
    }

    fn zero() -> Vec4 {
        // TODO: t really should be 1.0, for homogeneous coordinate.
        // But since we just ignore it at the moment, let's stick to 0s
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, PartialEq)]
struct Sphere {
    center: Vec4,
    radius: f32,
    _dead0: f32,
    _dead1: f32,
    _dead2: f32,
}

unsafe impl ocl::OclPrm for Sphere {}

impl std::ops::Add for Vec4 {
    type Output = Vec4;

    fn add(self, other: Vec4) -> Vec4 {
        Vec4 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}
impl std::ops::Sub for Vec4 {
    type Output = Vec4;

    fn sub(self, other: Vec4) -> Vec4 {
        Vec4 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl std::ops::Div<f32> for Vec4 {
    type Output = Vec4;

    fn div(self, scalar: f32) -> Vec4 {
        Vec4 {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
            w: self.w / scalar,
        }
    }
}

impl std::ops::Mul<f32> for Vec4 {
    type Output = Vec4;

    fn mul(self, scalar: f32) -> Vec4 {
        Vec4 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl From<Vec<f32>> for Vec4 {
    fn from(value: Vec<f32>) -> Self {
        let mut v = Vec4::zero();
        if value.len() >= 1 {
            v.x = value[0];
        }
        if value.len() >= 2 {
            v.y = value[1];
        }
        if value.len() >= 3 {
            v.z = value[2];
        }
        if value.len() >= 4 {
            v.w = value[3];
        }
        v
    }
}

impl From<json::Sphere> for Sphere {
    fn from(value: json::Sphere) -> Sphere {
        Sphere {
            center: value.center.into(),
            radius: value.radius,
            _dead0: 0.0,
            _dead1: 0.0,
            _dead2: 0.0,
        }
    }
}

fn mk_spheres(json: json::World) -> Vec<Sphere> {
    json.shapes
        .into_iter()
        .map(|shape| match shape {
            json::Shape::Sphere(s) => s.into(),
        })
        .collect()
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, PartialEq)]
struct Camera {
    samples_per_pixel: u32,
    max_depth: u32,
    image_width: u32,
    image_height: u32,
    center: Vec4,
    pixel00_loc: Vec4,
    pixel_delta_u: Vec4,
    pixel_delta_v: Vec4,
    aspect_ratio: f32,
}
unsafe impl ocl::OclPrm for Camera {}

impl Camera {
    fn new(aspect_ratio: f32, image_width: u32, samples_per_pixel: u32, max_depth: u32) -> Camera {
        let width = image_width as f32;
        let mut height = width / aspect_ratio;
        height = if height < 1.0 { 1.0 } else { height };
        let image_height = height as u32;

        let center = Vec4::zero();

        // Determine viewport dimensions.
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (width / height);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = Vec4::new(viewport_width, 0.0, 0.0, 0.0);
        let viewport_v = Vec4::new(0.0, -viewport_height, 0.0, 0.0);

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / width;
        let pixel_delta_v = viewport_v / height;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left: Vec4 = center
            - Vec4::new(0.0, 0.0, focal_length, 0.0)
            - (viewport_u / 2.0)
            - (viewport_v / 2.0);
        let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

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

    pub fn dump_info(&self) {
        println!("image width: {}", self.image_width);
        println!("image height: {}", self.image_height);
        println!("aspect ratio: {}", self.aspect_ratio);
        println!("samples per pixel: {}", self.samples_per_pixel);
        println!("max depth: {}", self.max_depth);
    }

    fn process_img(&self, img: Vec<Vec4>) -> Vec<u8> {
        let mut res = Vec::new();
        let factor: f32 = 1.0 / self.samples_per_pixel as f32;

        for j in 0..self.image_height as usize {
            for i in 0..self.image_width as usize {
                let mut c = Vec4::zero();

                let ndx = (i + j * self.image_width as usize) * self.samples_per_pixel as usize;
                for s in 0..self.samples_per_pixel as usize {
                    c = c + img[ndx + s];
                }
                c = c * factor;

                if !c.x.is_finite() || !c.y.is_finite() || !c.z.is_finite() {
                    eprintln!("/!\\ Pixel {} {} : {:?}", i, j, c);
                }
                let r = (linear_to_gamma(c.x) * 255.999) as u8;
                let g = (linear_to_gamma(c.y) * 255.999) as u8;
                let b = (linear_to_gamma(c.z) * 255.999) as u8;
                assert!(c.w == 0.0);

                res.push(r);
                res.push(g);
                res.push(b);
            }
        }
        res
    }
}

fn linear_to_gamma(linear_component: f32) -> f32 {
    f32::sqrt(linear_component)
}

fn main() -> Result<()> {
    println!("Camera {}", std::mem::size_of::<Camera>());
    println!("Sphere {}", std::mem::size_of::<Sphere>());
    println!("Vec4 {}", std::mem::size_of::<Vec4>());

    // Camera 84
    // Sphere 32
    // Vec4 16
    let cli = cli::Cli::parse();

    let width = cli.width.unwrap_or(IMAGE_WIDTH) as u32;
    let aspect_ratio = cli.aspect_ratio.unwrap_or(ASPECT_RATIO);
    let samples_per_pixel = cli.samples_per_pixel.unwrap_or(10) as u32;
    let max_depth = cli.max_depth.unwrap_or(10) as u32;

    let camera = Camera::new(aspect_ratio, width, samples_per_pixel, max_depth);

    if cli.dump_info {
        camera.dump_info()
    }

    // TODO: support windows utf16 bullshit
    let kernel = std::fs::read_to_string("./opencl/camera.cl")?;
    let scene_description = std::fs::read_to_string(r"./scene.json")?;

    let jworld: json::World =
        serde_json::from_str(&scene_description).context("Failed to read json input")?;

    for (name, mat) in jworld.materials.iter() {
        println!("material {}: {:?}", name, mat)
    }
    let spheres = mk_spheres(jworld);

    let pro_que = ProQue::builder()
        .src(kernel)
        .dims([
            camera.image_width,
            camera.image_height,
            camera.samples_per_pixel,
        ])
        .build()?;

    println!("pro_que dims: {:?}", pro_que.dims());
    println!("pro_que device: {:?}", pro_que.device());

    println!("create img buffer");
    let img: ocl::Buffer<Vec4> = pro_que
        .buffer_builder()
        .len(camera.image_width * camera.image_height * camera.samples_per_pixel)
        .build()?;
    let nr_spheres: u32 = spheres.len() as u32;
    println!("create sphere buffer");
    println!("sphere 0: {:?} {}", spheres[0].center, spheres[0].radius);
    println!("sphere 1: {:?} {}", spheres[1].center, spheres[1].radius);
    println!("sphere 2: {:?} {}", spheres[2].center, spheres[2].radius);
    println!("sphere 3: {:?} {}", spheres[3].center, spheres[3].radius);

    let buf_spheres: ocl::Buffer<Sphere> = pro_que
        .buffer_builder()
        .len(nr_spheres)
        .copy_host_slice(&spheres)
        .build()?;

    let buf_cam: ocl::Buffer<Camera> = pro_que
        .buffer_builder()
        .len(1)
        .copy_host_slice(&vec![camera])
        .build()?;

    let mut rng = rand::thread_rng();
    let seed0: u32 = rng.gen();
    let seed1: u32 = rng.gen();
    let seed2: u32 = rng.gen();
    let seed3: u32 = rng.gen();

    println!("Setting kernel arguments");
    let kernel = pro_que
        .kernel_builder("trace")
        .arg(&img)
        .arg(&buf_spheres)
        .arg(&buf_cam)
        .arg(&nr_spheres)
        .arg(&seed0)
        .arg(&seed1)
        .arg(&seed2)
        .arg(&seed3)
        .build()?;

    unsafe {
        kernel.enq()?;
    }

    let mut vec = vec![Vec4::zero(); img.len()];
    img.read(&mut vec).enq()?;

    println!("camera.center:        {:?}", camera.center);
    println!("camera.pixel00_loc:   {:?}", camera.pixel00_loc);
    println!("camera.pixel_delta_u: {:?}", camera.pixel_delta_u);
    println!("camera.pixel_delta_v: {:?}", camera.pixel_delta_v);

    // for i in 80000..80100 {
    for i in 0..4 {
        println!("{:?} {:?}", vec[10 * i + 0], vec[10 * i + 1]);
        println!("{:?} {:?}", vec[10 * i + 2], vec[10 * i + 3]);
        println!("{:?} {:?}", vec[10 * i + 4], vec[10 * i + 5]);
        println!("{:?} {:?}", vec[10 * i + 6], vec[10 * i + 7]);
        println!("{:?} {:?}", vec[10 * i + 8], vec[10 * i + 8]);
        println!()
    }

    let data = camera.process_img(vec);

    // println!("data: {:?}", &data[0..300]);

    // Save as PNG
    let img = match image::RgbImage::from_vec(
        camera.image_width as u32,
        camera.image_height as u32,
        data,
    ) {
        Some(img) => img,
        None => bail!("Failed to create RGB image"),
    };

    let img = image::DynamicImage::ImageRgb8(img);
    let path = Path::new(r"./image.png");
    img.save(path).context("Failed to save PNG image")?;

    Ok(())
}
