struct camera {
    unsigned int samples_per_pixel;
    unsigned int max_depth;
    unsigned int image_width;
    unsigned int image_height;
    float4 center;
    float4 pixel00_loc;
    float4 pixel_delta_u;
    float4 pixel_delta_v;
    float aspect_radio;
};

// TODO add rng
float4 camera_pixel_sample_square(struct camera *cam) {
    float offset = 0.42f; // TODO rng between [0.0, 1.0)

    float px = -0.5f + offset;
    float py = -0.5f + offset;

    return (px * cam->pixel_delta_u) + (py * cam->pixel_delta_v);
}

struct ray {
    float4 origin;
    float4 direction;
};

struct ray get_ray(struct camera *cam,
    unsigned int i,
    unsigned int j) {
    float x = (float)i;
    float y = (float)j;

    float4 pixel_center = cam->pixel00_loc
      + x * cam->pixel_delta_u
      + y * cam->pixel_delta_v;

    float4 pixel_sample = pixel_center + camera_pixel_sample_square(cam);
    float4 direction = pixel_sample - cam->center;

    struct ray r;
    r.origin = cam->center;
    r.direction = direction;

    return r;
}

float4 lerp(float4 a, float4 b, float t) {
    return a + t * (b - a);
}

struct sphere {
    float4 center;
    float radius;
    float _dead0;
    float _dead1;
    float _dead2;
};

int solveQuadratic(float a, float b, float c, float *t0, float *t1)
{
    float discr = b * b - 4 * a * c;
    if (discr < 0) {
        return 0;
    }

    if (fabs(discr) < FLT_EPSILON) {
        float t = -0.5f * b / a;
        *t0 = t;
        *t1 = t;
        return 1;
    }

    float q;
    if (b > 0) {
        q = -0.5f * (b + sqrt(discr));
    } else {
        q = -0.5f * (b - sqrt(discr));
    }

    float x0 = q / a;
    float x1 = c / q;

    if (x0 > x1) {
        float x = x0;
        x0 = x1;
        x1 = x;
    }

    *t0 = x0;
    *t1 = x1;
    
    return 2;
}

bool intersectRaySphere(const struct ray *r, __global const struct sphere *s, float *t) {
    // be careful that .w = 0 everywhere
    float4 v = r->origin - s->center;

    float a = dot(r->direction, r->direction);
    float b = 2 * dot(r->direction, v);
    float c = dot(v, v) - (s->radius * s->radius);

    float t0, t1;
    int ret = solveQuadratic(a, b, c, &t0, &t1);
    if (ret == 0) {
        return false;
    }

    if (ret == 1) {
        *t = t0;
        return true;
    }

    // assert(ret == 2)
    if (t0 < 0) {
       t0 = t1;
       if (t0 < 0) {
         return false;
       }
    }
    *t = t0;
    return true;
}

// TEMP
float4 sphere_color(unsigned int i) {
    if (i == 0) {
        return (float4)(0.8f, 0.8f, 0.0f, 0.0f);
    } else if (i == 1) {
        return (float4)(0.8f, 0.7f, 0.3f, 0.3f);
    } else if (i == 2) {
        return (float4)(0.8f, 0.8f, 0.8f, 0.8f);
    } else {
        return (float4)(0.8f, 0.8f, 0.6f, 0.2f);
    }
}

float4 ray_color(struct ray r,
    float4 attenuation,
    __global struct sphere *world,
    unsigned int nr_spheres,
    unsigned int depth) {

    if (depth == 0) {
        return (float4)(0.0f, 1.0f, 0.0f, 0.0f);
    }

    float background_gradient = 0.5f * (r.direction.y + 1.0f);
    float4 white = (float4)(1.0f, 1.0f, 1.0f, 0.0f);
    float4 red = (float4)(1.0f, 0.0f, 0.0f, 0.0f);
    float4 blue = (float4)(0.5f, 0.7f, 1.0f, 0.0f);

    unsigned int i;
    float closest = 1000000.0f; // infinity
    bool hit = false;
    float4 normal;
    float4 pos;
    unsigned int hit_nr;
    for (i = 0; i < nr_spheres; i++) {
        float t;
        __global const struct sphere *s = &world[i];
        if (intersectRaySphere(&r, s, &t)) {
            hit = true;
            if (t < closest) {
                closest = t;
                hit_nr = i;
                pos = r.origin + t * r.direction;
                normal = normalize(pos - s->center);
            }
        }
    }

    if (!hit) {
        // No hit, let's have a nice background for now
        return lerp(white, blue, background_gradient); 
    }

    // Simple Lambertian without rng
    attenuation = attenuation * sphere_color(hit_nr);
    struct ray scattered;
    scattered.origin = pos;
    scattered.direction = normal;
    return ray_color(scattered, attenuation, world, nr_spheres, depth - 1);
}

// No RNG for now
// No Recursion for now
__kernel void trace(__global float4 *img,
    __global struct sphere *spheres,
    struct camera cam,
    unsigned int nr_spheres) {

    int i = get_global_id(0);
    int j = get_global_id(1);
    int sample = get_global_id(2);

    // ray
    struct ray r = get_ray(&cam, i, j);
    float4 attenuation = (float4)(1.0f, 1.0f, 1.0f, 1.0f);
    float4 pixel_color = ray_color(r, attenuation, spheres, nr_spheres, 1); // cam.max_depth);

    unsigned int pos = i + cam.image_width * j;
    pos = sample + cam.samples_per_pixel * pos;

    img[pos] = pixel_color;
}
