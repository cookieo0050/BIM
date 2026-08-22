struct Camera {
    pos: vec3<f32>,
    _pad1: f32,
    dir: vec3<f32>,
    _pad2: f32,
    up: vec3<f32>,
    _pad3: f32,
    resolution: vec2<f32>,
    sphere_count: u32,
    frame_index: u32,
    fov_degrees: f32,
    max_bounces: u32,
    samples_per_pixel: u32,
    aperture: f32,
    focus_distance: f32,
    dof_enabled: u32,
    sun_enabled: u32,
    sun_intensity: f32,
    sun_dir: vec3<f32>,
    sun_angular_radius: f32,
    sun_color: vec3<f32>,
    floor_enabled: u32,
    floor_color: vec3<f32>,
    floor_roughness: f32,
    studio_top: vec3<f32>,
    _pad4: f32,
    studio_bottom: vec3<f32>,
    _pad5: f32,
    fog_color: vec3<f32>,
    fog_enabled: u32,
    _pad6: vec3<f32>,
    fog_density: f32,
    fill_dir: vec3<f32>,
    fill_intensity: f32,
    fill_color: vec3<f32>,
    fill_enabled: u32,
    skybox_rotation: f32,
    firefly_clamp: f32,
    floor_grid: u32,
    camera_roll: f32,
    ambient_color: vec3<f32>,
    ambient_intensity: f32,
    sky_intensity: f32,
    floor_height: f32,
    sun_shadows: u32,
    _pad7: u32,
    floor_metallic: f32,
    floor_ior: f32,
    floor_grid_scale: f32,
    floor_grid_thickness: f32,
    floor_checker: u32,
    floor_uv_scale: f32,
    floor_emissive_intensity: f32,
    _pad8: f32,
    floor_grid_color: vec3<f32>,
    _pad9: f32,
    floor_emissive: vec3<f32>,
    _pad10: f32,
};

struct PrimitiveData {
    shape_type: u32,
    _pad_a: u32, _pad_b: u32, _pad_c: u32,
    position: vec3<f32>,
    _pad_d: f32,
    size: vec3<f32>,
    _pad_e: f32,
    rotation: vec3<f32>,
    _pad_f: f32,
    albedo: vec3<f32>,
    _pad_g: f32,
    emissive: vec3<f32>,
    roughness: f32,
    metallic: f32,
    ior: f32,
    opacity: f32,
    flags: u32,
    texture_id: u32,
    uv_scale_x: f32,
    uv_scale_y: f32,
    uv_offset_x: f32,
    uv_offset_y: f32,
    clearcoat: f32,
    sheen: f32,
    transmission: f32,
    emissive_intensity: f32,
    specular_tint: f32,
    _pad_h: vec2<f32>,
};

struct SkyboxSettings {
    color: vec3<f32>,
    mode: u32,
};

@group(0) @binding(0) var<storage, read_write> accum_buffer: array<vec4<f32>>;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(0) @binding(2) var<storage, read> primitives: array<PrimitiveData>;
@group(0) @binding(3) var<uniform> skybox: SkyboxSettings;
@group(0) @binding(4) var env_texture: texture_2d<f32>;
@group(0) @binding(5) var env_sampler: sampler;
@group(0) @binding(6) var entity_atlas: texture_2d_array<f32>;
@group(0) @binding(7) var entity_sampler: sampler;

const PI: f32 = 3.14159265359;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

struct HitRecord {
    hit: u32,
    t: f32,
    point: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    albedo: vec3<f32>,
    emissive: vec3<f32>,
    roughness: f32,
    metallic: f32,
    ior: f32,
    opacity: f32,
    tex_id: u32,
    uv_scale: vec2<f32>,
    uv_offset: vec2<f32>,
    clearcoat: f32,
    sheen: f32,
    transmission: f32,
    emissive_intensity: f32,
    specular_tint: f32,
};

fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_f32(seed: ptr<function, u32>) -> f32 {
    *seed = pcg_hash(*seed);
    return f32(*seed) / 4294967295.0;
}

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn build_onb(n: vec3<f32>, b1: ptr<function, vec3<f32>>, b2: ptr<function, vec3<f32>>) {
    let sign_z = select(-1.0, 1.0, n.z >= 0.0);
    let a = -1.0 / (sign_z + n.z);
    let b = n.x * n.y * a;
    *b1 = vec3<f32>(1.0 + sign_z * n.x * n.x * a, sign_z * b, -sign_z * n.x);
    *b2 = vec3<f32>(b, sign_z + n.y * n.y * a, -n.y);
}

fn sample_cosine_hemisphere(n: vec3<f32>, seed: ptr<function, u32>) -> vec3<f32> {
    let u1 = rand_f32(seed);
    let u2 = rand_f32(seed);
    let r = sqrt(u1);
    let phi = 2.0 * PI * u2;

    let x = r * cos(phi);
    let y = r * sin(phi);
    let z = sqrt(max(0.0, 1.0 - u1));

    var b1: vec3<f32>;
    var b2: vec3<f32>;
    build_onb(n, &b1, &b2);

    return normalize(x * b1 + y * b2 + z * n);
}

fn sample_ggx(n: vec3<f32>, roughness: f32, seed: ptr<function, u32>) -> vec3<f32> {
    let u1 = rand_f32(seed);
    let u2 = rand_f32(seed);

    let a = max(roughness * roughness, 0.002);
    let phi = 2.0 * PI * u1;
    let cos_theta = sqrt((1.0 - u2) / (1.0 + (a * a - 1.0) * u2));
    let sin_theta = sqrt(max(0.0, 1.0 - cos_theta * cos_theta));

    let h_local = vec3<f32>(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);

    var b1: vec3<f32>;
    var b2: vec3<f32>;
    build_onb(n, &b1, &b2);

    return normalize(h_local.x * b1 + h_local.y * b2 + h_local.z * n);
}

fn rotate_y(v: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(v.x * c + v.z * s, v.y, -v.x * s + v.z * c);
}

fn rotate_x(v: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(v.x, v.y * c - v.z * s, v.y * s + v.z * c);
}

fn rotate_z(v: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(v.x * c - v.y * s, v.x * s + v.y * c, v.z);
}

fn rotate_point(p: vec3<f32>, rot: vec3<f32>) -> vec3<f32> {
    var result = p;
    result = rotate_x(result, rot.x);
    result = rotate_y(result, rot.y);
    result = rotate_z(result, rot.z);
    return result;
}

fn rotate_normal(n: vec3<f32>, rot: vec3<f32>) -> vec3<f32> {
    return normalize(rotate_point(n, rot));
}

fn fill_material(rec: ptr<function, HitRecord>, p: PrimitiveData) {
    (*rec).albedo = p.albedo;
    (*rec).emissive = p.emissive;
    (*rec).roughness = p.roughness;
    (*rec).metallic = p.metallic;
    (*rec).ior = p.ior;
    (*rec).opacity = p.opacity;
    (*rec).tex_id = p.texture_id;
    (*rec).uv_scale = vec2<f32>(p.uv_scale_x, p.uv_scale_y);
    (*rec).uv_offset = vec2<f32>(p.uv_offset_x, p.uv_offset_y);
    (*rec).clearcoat = p.clearcoat;
    (*rec).sheen = p.sheen;
    (*rec).transmission = p.transmission;
    (*rec).emissive_intensity = p.emissive_intensity;
    (*rec).specular_tint = p.specular_tint;
}

fn hit_sphere(p: PrimitiveData, r: Ray, t_min: f32, t_max: f32) -> HitRecord {
    var rec: HitRecord;
    rec.hit = 0u;

    let oc = r.origin - p.position;
    let a = dot(r.direction, r.direction);
    let half_b = dot(oc, r.direction);
    let radius = p.size.x;
    let c = dot(oc, oc) - radius * radius;
    let discriminant = half_b * half_b - a * c;

    if (discriminant > 0.0) {
        let sqrtd = sqrt(discriminant);
        var root = (-half_b - sqrtd) / a;

        if (root < t_min || root > t_max) {
            root = (-half_b + sqrtd) / a;
        }

        if (root >= t_min && root <= t_max) {
            rec.hit = 1u;
            rec.t = root;
            rec.point = r.origin + r.direction * rec.t;
            let outward_normal = (rec.point - p.position) / radius;
            rec.normal = outward_normal;

            let theta = acos(clamp(outward_normal.y, -1.0, 1.0));
            let phi = atan2(outward_normal.z, outward_normal.x);
            rec.uv = vec2<f32>(1.0 - (phi + PI) / (2.0 * PI), 1.0 - theta / PI);

            fill_material(&rec, p);
        }
    }

    return rec;
}

fn hit_cube(p: PrimitiveData, r: Ray, t_min: f32, t_max: f32) -> HitRecord {
    var rec: HitRecord;
    rec.hit = 0u;

    let inv_rot = -p.rotation;
    let local_origin = rotate_point(r.origin - p.position, inv_rot);
    let local_dir = rotate_point(r.direction, inv_rot);

    let half_extents = p.size;
    let inv_dir = vec3<f32>(
        select(1.0e30, 1.0 / local_dir.x, abs(local_dir.x) > 0.00001),
        select(1.0e30, 1.0 / local_dir.y, abs(local_dir.y) > 0.00001),
        select(1.0e30, 1.0 / local_dir.z, abs(local_dir.z) > 0.00001)
    );

    let t1 = (-half_extents - local_origin) * inv_dir;
    let t2 = (half_extents - local_origin) * inv_dir;

    let tmin = max(max(min(t1.x, t2.x), min(t1.y, t2.y)), min(t1.z, t2.z));
    let tmax = min(min(max(t1.x, t2.x), max(t1.y, t2.y)), max(t1.z, t2.z));

    if (tmax >= tmin && tmax > 0.001) {
        var root = tmin;
        if (root < t_min) { root = tmax; }
        if (root >= t_min && root <= t_max) {
            rec.hit = 1u;
            rec.t = root;
            rec.point = r.origin + r.direction * rec.t;

            let local_hit = local_origin + local_dir * root;
            var local_normal = vec3<f32>(0.0);
            var face_uv = vec2<f32>(0.0);
            let eps = 0.001;

            if (abs(local_hit.x - half_extents.x) < eps) {
                local_normal = vec3<f32>(1.0, 0.0, 0.0);
                face_uv = vec2<f32>((local_hit.z / half_extents.z + 1.0) * 0.5, (local_hit.y / half_extents.y + 1.0) * 0.5);
            } else if (abs(local_hit.x + half_extents.x) < eps) {
                local_normal = vec3<f32>(-1.0, 0.0, 0.0);
                face_uv = vec2<f32>((1.0 - local_hit.z / half_extents.z) * 0.5, (local_hit.y / half_extents.y + 1.0) * 0.5);
            } else if (abs(local_hit.y - half_extents.y) < eps) {
                local_normal = vec3<f32>(0.0, 1.0, 0.0);
                face_uv = vec2<f32>((local_hit.x / half_extents.x + 1.0) * 0.5, (1.0 - local_hit.z / half_extents.z) * 0.5);
            } else if (abs(local_hit.y + half_extents.y) < eps) {
                local_normal = vec3<f32>(0.0, -1.0, 0.0);
                face_uv = vec2<f32>((local_hit.x / half_extents.x + 1.0) * 0.5, (local_hit.z / half_extents.z + 1.0) * 0.5);
            } else if (abs(local_hit.z - half_extents.z) < eps) {
                local_normal = vec3<f32>(0.0, 0.0, 1.0);
                face_uv = vec2<f32>((1.0 - local_hit.x / half_extents.x) * 0.5, (local_hit.y / half_extents.y + 1.0) * 0.5);
            } else if (abs(local_hit.z + half_extents.z) < eps) {
                local_normal = vec3<f32>(0.0, 0.0, -1.0);
                face_uv = vec2<f32>((local_hit.x / half_extents.x + 1.0) * 0.5, (local_hit.y / half_extents.y + 1.0) * 0.5);
            }

            rec.normal = rotate_normal(local_normal, p.rotation);
            rec.uv = face_uv;
            fill_material(&rec, p);
        }
    }

    return rec;
}

fn hit_cylinder(p: PrimitiveData, r: Ray, t_min: f32, t_max: f32) -> HitRecord {
    var rec: HitRecord;
    rec.hit = 0u;

    let inv_rot = -p.rotation;
    let local_origin = rotate_point(r.origin - p.position, inv_rot);
    let local_dir = rotate_point(r.direction, inv_rot);

    let radius = p.size.x;
    let half_height = p.size.y;

    let oc_xz = vec2<f32>(local_origin.x, local_origin.z);
    let dir_xz = vec2<f32>(local_dir.x, local_dir.z);

    let a = dot(dir_xz, dir_xz);
    let b = dot(oc_xz, dir_xz);
    let c = dot(oc_xz, oc_xz) - radius * radius;
    let discriminant = b * b - a * c;

    var best_t = 1.0e30;
    var hit_normal = vec3<f32>(0.0);
    var hit_uv = vec2<f32>(0.0);

    if (discriminant > 0.0) {
        let sqrtd = sqrt(discriminant);
        var root = (-b - sqrtd) / a;
        if (root >= t_min && root <= t_max) {
            let y = local_origin.y + local_dir.y * root;
            if (abs(y) <= half_height) {
                best_t = root;
                let hit_xz = vec2<f32>(local_origin.x + local_dir.x * root, local_origin.z + local_dir.z * root);
                hit_normal = vec3<f32>(hit_xz.x, 0.0, hit_xz.y) / radius;
                let angle = atan2(hit_xz.y, hit_xz.x);
                hit_uv = vec2<f32>((angle + PI) / (2.0 * PI), (y / half_height + 1.0) * 0.5);
            }
        }
        root = (-b + sqrtd) / a;
        if (root >= t_min && root <= t_max && root < best_t) {
            let y = local_origin.y + local_dir.y * root;
            if (abs(y) <= half_height) {
                best_t = root;
                let hit_xz = vec2<f32>(local_origin.x + local_dir.x * root, local_origin.z + local_dir.z * root);
                hit_normal = vec3<f32>(hit_xz.x, 0.0, hit_xz.y) / radius;
                let angle = atan2(hit_xz.y, hit_xz.x);
                hit_uv = vec2<f32>((angle + PI) / (2.0 * PI), (y / half_height + 1.0) * 0.5);
            }
        }
    }

    let cap_t_top = (half_height - local_origin.y) / local_dir.y;
    if (cap_t_top >= t_min && cap_t_top <= t_max && cap_t_top < best_t) {
        let hit_xz = vec2<f32>(local_origin.x + local_dir.x * cap_t_top, local_origin.z + local_dir.z * cap_t_top);
        if (dot(hit_xz, hit_xz) <= radius * radius) {
            best_t = cap_t_top;
            hit_normal = vec3<f32>(0.0, 1.0, 0.0);
            hit_uv = vec2<f32>((hit_xz.x / radius + 1.0) * 0.5, (hit_xz.y / radius + 1.0) * 0.5);
        }
    }

    let cap_t_bot = (-half_height - local_origin.y) / local_dir.y;
    if (cap_t_bot >= t_min && cap_t_bot <= t_max && cap_t_bot < best_t) {
        let hit_xz = vec2<f32>(local_origin.x + local_dir.x * cap_t_bot, local_origin.z + local_dir.z * cap_t_bot);
        if (dot(hit_xz, hit_xz) <= radius * radius) {
            best_t = cap_t_bot;
            hit_normal = vec3<f32>(0.0, -1.0, 0.0);
            hit_uv = vec2<f32>((hit_xz.x / radius + 1.0) * 0.5, (hit_xz.y / radius + 1.0) * 0.5);
        }
    }

    if (best_t < 1.0e30) {
        rec.hit = 1u;
        rec.t = best_t;
        rec.point = r.origin + r.direction * rec.t;
        rec.normal = rotate_normal(hit_normal, p.rotation);
        rec.uv = hit_uv;
        fill_material(&rec, p);
    }

    return rec;
}

fn hit_plane_prim(p: PrimitiveData, r: Ray, t_min: f32, t_max: f32) -> HitRecord {
    var rec: HitRecord;
    rec.hit = 0u;

    let normal = normalize(p.size);
    let denom = dot(normal, r.direction);
    if (abs(denom) < 0.0001) { return rec; }

    let t = dot(p.position - r.origin, normal) / denom;
    if (t >= t_min && t <= t_max) {
        rec.hit = 1u;
        rec.t = t;
        rec.point = r.origin + r.direction * t;
        rec.normal = normal;

        let local_point = rec.point - p.position;
        rec.uv = vec2<f32>(local_point.x * 0.5, local_point.z * 0.5);

        fill_material(&rec, p);
    }

    return rec;
}

fn hit_primitive(p: PrimitiveData, r: Ray, t_min: f32, t_max: f32) -> HitRecord {
    if (p.shape_type == 0u) {
        return hit_sphere(p, r, t_min, t_max);
    } else if (p.shape_type == 1u) {
        return hit_cube(p, r, t_min, t_max);
    } else if (p.shape_type == 2u) {
        return hit_cylinder(p, r, t_min, t_max);
    } else {
        return hit_plane_prim(p, r, t_min, t_max);
    }
}

fn hit_floor(r: Ray, t_min: f32, t_max: f32) -> HitRecord {
    var rec: HitRecord;
    rec.hit = 0u;

    if (camera.floor_enabled == 0u) { return rec; }
    if (abs(r.direction.y) < 0.0001) { return rec; }

    let t = (camera.floor_height - r.origin.y) / r.direction.y;
    if (t >= t_min && t <= t_max) {
        rec.hit = 1u;
        rec.t = t;
        rec.point = r.origin + r.direction * t;
        rec.normal = vec3<f32>(0.0, 1.0, 0.0);
        rec.uv = vec2<f32>(rec.point.x * 0.25, rec.point.z * 0.25) * camera.floor_uv_scale;
        rec.tex_id = 0u;

        let cell = max(camera.floor_grid_scale, 0.0001);
        let gp = rec.point.xz / cell;
        let grid = fract(gp);
        let is_line_raw = step(grid.x, camera.floor_grid_thickness) + step(grid.y, camera.floor_grid_thickness);
        let is_line = select(0.0, min(is_line_raw, 3.0), camera.floor_grid == 1u);

        let dist = length(rec.point.xz);
        let fog_factor = clamp(dist * 0.05, 0.0, 1.0);

        let base_floor = camera.floor_color;
        let grid_color = camera.floor_grid_color;
        var albedo = base_floor;
        albedo = mix(albedo, grid_color, is_line);

        let checker_sum = floor(gp.x) + floor(gp.y);
        let checker_on = (checker_sum - 2.0 * floor(checker_sum * 0.5)) > 0.5;
        if (camera.floor_checker == 1u && checker_on) {
            albedo = albedo * 0.82;
        }

        rec.albedo = mix(albedo, base_floor, fog_factor);
        rec.emissive = camera.floor_emissive * camera.floor_emissive_intensity;
        rec.roughness = camera.floor_roughness;
        rec.metallic = camera.floor_metallic;
        rec.ior = camera.floor_ior;
        rec.opacity = 1.0;
        rec.uv_scale = vec2<f32>(1.0) * camera.floor_uv_scale;
        rec.uv_offset = vec2<f32>(0.0);
        rec.clearcoat = 0.0;
        rec.sheen = 0.0;
        rec.transmission = 0.0;
        rec.emissive_intensity = 1.0;
        rec.specular_tint = 1.0;
    }

    return rec;
}

fn trace_scene(r: Ray, t_min: f32, t_max: f32) -> HitRecord {
    var closest_hit: HitRecord;
    closest_hit.hit = 0u;
    var closest_so_far = t_max;

    let floor_hit = hit_floor(r, t_min, closest_so_far);
    if (floor_hit.hit == 1u) {
        closest_so_far = floor_hit.t;
        closest_hit = floor_hit;
    }

    for (var i = 0u; i < camera.sphere_count; i++) {
        if (primitives[i].flags & 1u) == 0u { continue; }
        let hit = hit_primitive(primitives[i], r, t_min, closest_so_far);
        if (hit.hit == 1u) {
            closest_so_far = hit.t;
            closest_hit = hit;
        }
    }

    return closest_hit;
}

fn get_texture_color(hit: HitRecord) -> vec3<f32> {
    if (hit.tex_id == 0u) {
        return hit.albedo;
    }
    let layer = i32(hit.tex_id - 1u);
    let dims = textureDimensions(entity_atlas);
    let uv = hit.uv * hit.uv_scale + hit.uv_offset;
    let px = clamp(vec2<i32>(uv * vec2<f32>(dims)), vec2<i32>(0), vec2<i32>(dims) - vec2<i32>(1));
    let tex_color = textureLoad(entity_atlas, px, layer, 0).rgb;
    return hit.albedo * tex_color;
}

fn studio_environment(dir: vec3<f32>) -> vec3<f32> {
    let t = 0.5 * (dir.y + 1.0);
    var sky = mix(camera.studio_bottom, camera.studio_top, clamp(t, 0.0, 1.0));

    let key_spec = pow(max(dot(dir, vec3<f32>(-0.137, 0.366, -0.183)), 0.0), 16.0);
    sky += vec3<f32>(3.5, 3.2, 3.0) * key_spec;

    let fill_spec = pow(max(dot(dir, vec3<f32>(0.832, 0.312, 0.520)), 0.0), 12.0);
    sky += vec3<f32>(0.8, 1.0, 1.2) * fill_spec;

    return sky;
}

fn sample_equirect(dir: vec3<f32>) -> vec3<f32> {
    let theta = atan2(dir.z, dir.x);
    let phi = asin(clamp(dir.y, -1.0, 1.0));
    let u = 0.5 + theta / (2.0 * PI);
    let v = 0.5 - phi / PI;
    return textureSampleLevel(env_texture, env_sampler, vec2<f32>(u, v), 0.0).rgb;
}

fn sample_skybox(dir: vec3<f32>) -> vec3<f32> {
    var color: vec3<f32>;
    let rotated_dir = rotate_y(dir, camera.skybox_rotation);
    if (skybox.mode == 0u) {
        color = studio_environment(rotated_dir);
    } else if (skybox.mode == 1u) {
        color = skybox.color;
    } else {
        color = sample_equirect(rotated_dir);
    }

    color *= camera.sky_intensity;

    if (camera.sun_enabled == 1u) {
        let sun_dir = normalize(camera.sun_dir);
        let d = max(dot(normalize(dir), sun_dir), 0.0);
        let cos_r = cos(camera.sun_angular_radius);
        let cos_outer = cos(camera.sun_angular_radius * 2.0);
        let disk = smoothstep(cos_outer, cos_r, d);
        let glow = pow(d, 1500.0) * 0.5;
        color += camera.sun_color * camera.sun_intensity * (disk * 10.0 + glow);
    }

    return color;
}

fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (vec3<f32>(1.0) - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn sample_sun_direction(seed: ptr<function, u32>) -> vec3<f32> {
    var dir = normalize(camera.sun_dir);
    if (camera.sun_angular_radius > 0.001) {
        let u1 = rand_f32(seed);
        let u2 = rand_f32(seed);
        let cos_max = cos(camera.sun_angular_radius);
        let cos_theta = mix(cos_max, 1.0, u1);
        let sin_theta = sqrt(max(0.0, 1.0 - cos_theta * cos_theta));
        let phi = 2.0 * PI * u2;

        var b1: vec3<f32>;
        var b2: vec3<f32>;
        build_onb(dir, &b1, &b2);

        dir = normalize(sin_theta * cos(phi) * b1 + sin_theta * sin(phi) * b2 + cos_theta * dir);
    }
    return dir;
}

fn sample_sun_light(pos: vec3<f32>, n: vec3<f32>, seed: ptr<function, u32>) -> vec3<f32> {
    if (camera.sun_enabled == 0u) {
        return vec3<f32>(0.0);
    }

    let l = sample_sun_direction(seed);
    let ndotl = dot(n, l);
    if (ndotl <= 0.0) {
        return vec3<f32>(0.0);
    }

    if (camera.sun_shadows == 1u) {
        let shadow_ray = Ray(pos + n * 0.002, l);
        let shadow_hit = trace_scene(shadow_ray, 0.002, 1000.0);
        if (shadow_hit.hit == 1u) {
            return vec3<f32>(0.0);
        }
    }

    return camera.sun_color * camera.sun_intensity * ndotl;
}

fn sample_fill_light(n: vec3<f32>) -> vec3<f32> {
    if (camera.fill_enabled == 0u) {
        return vec3<f32>(0.0);
    }
    let l = normalize(camera.fill_dir);
    let ndotl = max(dot(n, l), 0.0);
    return camera.fill_color * camera.fill_intensity * ndotl;
}

fn trace_ray(start_ray: Ray, seed: ptr<function, u32>) -> vec3<f32> {
    var current_ray = start_ray;
    var accumulated_color = vec3<f32>(0.0);
    var throughput = vec3<f32>(1.0);

    for (var bounce = 0u; bounce < camera.max_bounces; bounce++) {
        let hit = trace_scene(current_ray, 0.001, 1000.0);

        if (hit.hit == 1u) {
            // Exponential distance fog along the segment
            if (camera.fog_enabled == 1u && camera.fog_density > 0.0) {
                let fog_amount = 1.0 - exp(-camera.fog_density * hit.t);
                accumulated_color += throughput * camera.fog_color * fog_amount;
                throughput *= exp(-camera.fog_density * hit.t);
            }

            accumulated_color += throughput * hit.emissive * hit.emissive_intensity;

            let V = -current_ray.direction;
            let N = hit.normal;

            let surface_albedo = get_texture_color(hit);

            // Direct sun lighting on diffuse-ish surfaces
            if (hit.transmission < 0.99) {
                accumulated_color += throughput * surface_albedo * sample_sun_light(hit.point, N, seed);
                accumulated_color += throughput * surface_albedo * sample_fill_light(N);
                accumulated_color += throughput * surface_albedo * camera.ambient_color * camera.ambient_intensity;
            }

            let F0 = mix(vec3<f32>(pow((hit.ior - 1.0) / (hit.ior + 1.0), 2.0)), surface_albedo * hit.specular_tint, hit.metallic);
            let cos_theta = max(dot(N, V), 0.0);
            let F = fresnel_schlick(cos_theta, F0);

            let r = rand_f32(seed);

            // ===== Glass / transmission lobe =====
            if (r < hit.transmission) {
                let entering = dot(current_ray.direction, N) < 0.0;
                let n_front = select(-N, N, entering);
                let cos_i = clamp(dot(-current_ray.direction, n_front), 0.0, 1.0);
                let eta = select(hit.ior, 1.0 / hit.ior, entering);
                let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);

                let f0_glass = pow((hit.ior - 1.0) / (hit.ior + 1.0), 2.0);
                var f_glass = f0_glass + (1.0 - f0_glass) * pow(1.0 - cos_i, 5.0);
                if (k < 0.0) { f_glass = 1.0; }

                var bounce_dir: vec3<f32>;
                if (rand_f32(seed) < f_glass) {
                    bounce_dir = reflect(current_ray.direction, n_front);
                } else {
                    bounce_dir = normalize(eta * current_ray.direction + (eta * cos_i - sqrt(max(k, 0.0))) * n_front);
                }

                throughput *= mix(vec3<f32>(1.0), surface_albedo, 0.6) / max(hit.transmission, 0.001);

                let offset_dir = select(-n_front, n_front, dot(bounce_dir, n_front) > 0.0);
                current_ray = Ray(hit.point + offset_dir * 0.001, bounce_dir);
            } else {
                let r2 = (r - hit.transmission) / max(1.0 - hit.transmission, 0.001);

                // Clearcoat: extra smooth white glossy lobe
                let cc_fres = fresnel_schlick(cos_theta, vec3<f32>(0.04)).r;
                let cc_chance = clamp(hit.clearcoat * cc_fres, 0.0, 0.95);

                let spec_chance = clamp(max(max(F.r, max(F.g, F.b)), 0.0), 0.05, 0.95);
                let coat_rough = min(hit.roughness, 0.08);

                if (r2 < cc_chance) {
                    let H = sample_ggx(N, coat_rough, seed);
                    var bounce_dir = -V - 2.0 * dot(-V, H) * H;
                    if (dot(bounce_dir, N) <= 0.0) { bounce_dir = reflect(V, N); }
                    throughput *= vec3<f32>(cc_fres * hit.clearcoat) / max(cc_chance, 0.001);

                    let offset_dir = select(-N, N, dot(bounce_dir, N) > 0.0);
                    current_ray = Ray(hit.point + offset_dir * 0.001, bounce_dir);
                } else if (r2 < cc_chance + spec_chance) {
                    let H = sample_ggx(N, hit.roughness, seed);
                    var bounce_dir = -V - 2.0 * dot(-V, H) * H;
                    if (dot(bounce_dir, N) <= 0.0) { bounce_dir = reflect(V, N); }
                    throughput *= F / max(spec_chance, 0.001);

                    let offset_dir = select(-N, N, dot(bounce_dir, N) > 0.0);
                    current_ray = Ray(hit.point + offset_dir * 0.001, bounce_dir);
                } else {
                    var bounce_dir = sample_cosine_hemisphere(N, seed);
                    let diffuse_weight = max(1.0 - spec_chance, 0.01);
                    throughput *= surface_albedo * (1.0 - hit.metallic) * (vec3<f32>(1.0) - F) / diffuse_weight;

                    // Sheen: retro-reflective fabric rim
                    let sheen_term = hit.sheen * pow(1.0 - cos_theta, 5.0);
                    throughput *= vec3<f32>(1.0 + sheen_term);

                    if (dot(bounce_dir, N) <= 0.0) {
                        bounce_dir = N;
                    }

                    let offset_dir = select(-N, N, dot(bounce_dir, N) > 0.0);
                    current_ray = Ray(hit.point + offset_dir * 0.001, bounce_dir);
                }
            }

            throughput = min(throughput, vec3<f32>(8.0));

            let p = max(throughput.r, max(throughput.g, throughput.b));
            if (p < 0.001) { break; }
            if (bounce > 0u) {
                if (rand_f32(seed) > p) { break; }
                throughput /= p;
            }
        } else {
            var sky_color = sample_skybox(current_ray.direction);
            if (camera.fog_enabled == 1u && camera.fog_density > 0.0) {
                let sky_fog = 1.0 - exp(-camera.fog_density * 40.0);
                sky_color = mix(sky_color, camera.fog_color, sky_fog);
            }
            accumulated_color += throughput * sky_color;
            break;
        }
    }

    let lum = luminance(accumulated_color);
    let clamp_max = max(camera.firefly_clamp, 0.5);
    if (lum > clamp_max) {
        accumulated_color *= clamp_max / lum;
    }

    return accumulated_color;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let resolution = vec2<u32>(camera.resolution);
    if (global_id.x >= resolution.x || global_id.y >= resolution.y) {
        return;
    }

    let screen_pos = vec2<f32>(global_id.xy);
    let aspect_ratio = camera.resolution.x / camera.resolution.y;

    let forward = normalize(camera.dir);
    let right = normalize(cross(camera.up, forward));
    let true_up = cross(forward, right);

    let fov_scale = tan(radians(camera.fov_degrees) * 0.5);

    var rng_seed = u32(global_id.x) * 1973u + u32(global_id.y) * 9277u + camera.frame_index * 6193u + 1121u;

    let spp = max(camera.samples_per_pixel, 1u);
    var final_color = vec3<f32>(0.0);

    for (var s = 0u; s < spp; s++) {
        let rx = rand_f32(&rng_seed);
        let ry = rand_f32(&rng_seed);

        let jitter = vec2<f32>(rx - 0.5, ry - 0.5) / camera.resolution;
        let uv = ((screen_pos / camera.resolution) + jitter) * 2.0 - 1.0;

        var ray_dir = normalize(
            forward +
            right * (uv.x * aspect_ratio * fov_scale) -
            true_up * (uv.y * fov_scale)
        );
        var ray_origin = camera.pos;

        // Depth of field: thin lens model
        if (camera.dof_enabled == 1u && camera.aperture > 0.0) {
            let focal_dist = max(camera.focus_distance, 0.01);
            let focal_point = camera.pos + ray_dir * (focal_dist / max(dot(ray_dir, forward), 0.001));

            let lu = rand_f32(&rng_seed);
            let lv = rand_f32(&rng_seed);
            let lens_r = camera.aperture * sqrt(lu);
            let lens_a = 2.0 * PI * lv;
            let offset = right * cos(lens_a) * lens_r + true_up * sin(lens_a) * lens_r;

            ray_origin = camera.pos + offset;
            ray_dir = normalize(focal_point - ray_origin);
        }

        let ray = Ray(ray_origin, ray_dir);
        final_color += trace_ray(ray, &rng_seed);
    }

    final_color /= f32(spp);

    let buf_idx = global_id.y * u32(camera.resolution.x) + global_id.x;
    let prev = accum_buffer[buf_idx].rgb;

    var accumulated: vec3<f32>;
    if (camera.frame_index == 0u) {
        accumulated = final_color;
    } else {
        let t = 1.0 / f32(camera.frame_index + 1u);
        accumulated = mix(prev, final_color, t);
    }

    accum_buffer[buf_idx] = vec4<f32>(accumulated, 1.0);
}
