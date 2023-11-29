///////// /////// /////// /////// ///////
// Simple Xorshift RNG

uint xorshift_int(uint4* ctx) {
	uint t = ctx->x ^ (ctx->x << 11);
	*ctx = ctx->yzww;
	ctx->w = ctx->w ^ (ctx->w >> 19) ^ (t ^ (t >> 8));

	return ctx->w;
}

// roughly inside [0.0, 1.0)
float xorshift_float(uint4* ctx) {
	return xorshift_int(ctx) * 2.3283064e-10;
}

// BETWEEN -1 and 1 !!!
float4 random_vector_rang(uint4 *ctx) {
    float x = 0.5f; // (xorshift_float(ctx) * 2.0f) - 1.0f;
    float y = -0.4f; // (xorshift_float(ctx) * 2.0f) - 1.0f;
    float z = 0.32f; // (xorshift_float(ctx) * 2.0f) - 1.0f;
    return (float4)(x, y, z, 0.0f);
}

float4 random_vector_in_unit_sphere(uint4* ctx) {
    float4 v;
    for (int i = 0; i < 1000; i++) {
        v = random_vector_rang(ctx);
        float d = v.x * v.x + v.y * v.y + v.z * v.z;
        if (d < FLT_EPSILON) {
            return v;
        }
    }
    return v;
}

float4 random_unit_vector(uint4 *ctx) {
    float4 v = random_vector_in_unit_sphere(ctx);
    return normalize(v);
}

bool vector_near_zero(float4 *v) {
    bool x = fabs(v->x) < FLT_EPSILON;
    bool y = fabs(v->y) < FLT_EPSILON;
    bool z = fabs(v->z) < FLT_EPSILON;
    return (x && y && z);
}
///////// /////// /////// /////// ///////

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
float4 camera_pixel_sample_square(__global struct camera *cam, uint4* ctx) {
    float offset = 0.4f; // xorshift_float(ctx);

    float px = -0.5f + offset;
    float py = -0.5f + offset;

    return (px * cam->pixel_delta_u) + (py * cam->pixel_delta_v);
}

struct ray {
    float4 origin;
    float4 direction;
};

struct ray get_ray(__global struct camera *cam,
    uint4* ctx,
    unsigned int i,
    unsigned int j) {
    float x = (float)i;
    float y = (float)j;

    float4 pixel_center = cam->pixel00_loc
      + x * cam->pixel_delta_u
      + y * cam->pixel_delta_v;

    float4 pixel_sample = pixel_center + camera_pixel_sample_square(cam, ctx);
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
        return (float4)(0.7f, 0.3f, 0.3f, 0.0f);
    } else if (i == 2) {
        return (float4)(0.8f, 0.8f, 0.8f, 0.0f);
    } else {
        return (float4)(0.8f, 0.6f, 0.2f, 0.0f);
    }
}

struct intersection {
    float toi;
    float4 normal;
    float4 attenuation;
};

bool intersectSphere(struct ray *r,
    __global struct sphere *world,
    unsigned int nr_spheres,
    struct intersection *info) {
    float closest;
    closest = 1000000.0f; // infinity
    bool hit = false;

    unsigned int i;
    for (i = 0; i < nr_spheres; i++) {
        float toi = 0.0f;
        __global const struct sphere *s = &world[i];
        if (intersectRaySphere(r, s, &toi)) {
            hit = true;
            if (toi < closest) {
                closest = toi;
                info->toi = toi;
                float4 pos = r->origin + toi * r->direction;
                info->normal = normalize(pos - s->center);
                info->attenuation = sphere_color(i);
            }
        }
    }

    return hit;
}

float4 ray_color(uint4* ctx,
    struct ray ray0,
    __global struct sphere *world,
    unsigned int nr_spheres,
    unsigned int depth) {

    float4 white = (float4)(1.0f, 1.0f, 1.0f, 0.0f);
    float4 blue = (float4)(0.5f, 0.7f, 1.0f, 0.0f);

    struct ray r = ray0;
    float4 color = (float4)(1.0f, 1.0f, 1.0f, 0.0f);

    while (depth > 0) {
        float background_gradient = 0.5f * (r.direction.y + 1.0f);
        struct intersection info;

        if (intersectSphere(&r, world, nr_spheres, &info)) {
            float4 scatter_direction = info.normal + random_unit_vector(ctx);
            if (vector_near_zero(&scatter_direction)) {
                scatter_direction = info.normal;
            }
            float toi = info.toi - 0.1f;
            float4 scatter_origin = r.origin + toi * r.direction;
            // float4 res = info.normal + (float4)(1.0f, 1.0f, 1.0f, 0.0f);
            // return (res / 2);
            r.origin = scatter_origin;
            r.direction = scatter_direction;
            color = color * info.attenuation;
            depth -= 1;
        } else {
            // No hit, let's have a nice background for now
            float4 bg = lerp(white, blue, background_gradient);
            return color * bg;
        }
    }

    return (float4)(0.0f, 0.0f, 0.0f, 0.0f);
}

// No RNG for now
// No Recursion for now
__kernel void trace(__global float4 *img,
    __global struct sphere *spheres,
    __global struct camera *cam,
    unsigned int nr_spheres,
    uint seed0,
    uint seed1,
    uint seed2,
    uint seed3) {

    uint4 ctx = (uint4)(seed0, seed1, seed2, seed3);

    int i = get_global_id(0);
    int j = get_global_id(1);
    int sample = get_global_id(2);

    struct ray r = get_ray(cam, &ctx, i, j);
    float4 pixel_color = ray_color(&ctx, r, spheres, nr_spheres, cam->max_depth);

    unsigned int pos = i + cam->image_width * j;
    pos = sample + cam->samples_per_pixel * pos;

    img[pos] = pixel_color;
}
