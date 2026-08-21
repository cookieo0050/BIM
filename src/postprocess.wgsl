struct Params {
    bloom_threshold: f32,
    bloom_soft_threshold: f32,
    bloom_intensity: f32,
    bloom_use_karis: u32,
    bloom_luminance_bias: f32,
    bloom_downsample_delta: f32,
    bloom_upsample_delta: f32,
    bloom_blend_mode: u32,
    exposure: f32,
    temperature: f32,
    tint: f32,
    contrast: f32,
    linear_midpoint: f32,
    brightness: f32,
    filter_intensity: f32,
    saturation: f32,
    color_filter_r: f32,
    color_filter_g: f32,
    color_filter_b: f32,
    vignette_intensity: f32,
    vignette_roundness: f32,
    vignette_smoothness: f32,
    vignette_color_r: f32,
    vignette_color_g: f32,
    vignette_color_b: f32,
    vignette_size_x: f32,
    vignette_size_y: f32,
    vignette_offset_x: f32,
    vignette_offset_y: f32,
    ca_intensity: f32,
    ca_hardness: f32,
    ca_offset_r: f32,
    ca_offset_g: f32,
    ca_offset_b: f32,
    ca_focal_offset_x: f32,
    ca_focal_offset_y: f32,
    ca_radius_x: f32,
    ca_radius_y: f32,
    grain_intensity: f32,
    grain_response: f32,
    dither_spread: f32,
    dither_color_count_r: f32,
    dither_color_count_g: f32,
    dither_color_count_b: f32,
    sharpen_amount: f32,
    sharpen_radius: f32,
    pixelate_size: f32,
    scanline_intensity: f32,
    scanline_frequency: f32,
    edge_intensity: f32,
    edge_threshold: f32,
    edge_color_r: f32,
    edge_color_g: f32,
    edge_color_b: f32,
    distortion_k1: f32,
    distortion_k2: f32,
    letterbox_amount: f32,
    halation_intensity: f32,
    halation_color_r: f32,
    halation_color_g: f32,
    halation_color_b: f32,
    radial_intensity: f32,
    radial_center_x: f32,
    radial_center_y: f32,
    hue_shift_degrees: f32,
    lift_r: f32,
    lift_g: f32,
    lift_b: f32,
    gain_r: f32,
    gain_g: f32,
    gain_b: f32,
    gamma_value: f32,
    posterize_levels: f32,
    glitch_intensity: f32,
    glitch_speed: f32,
    zoom_factor: f32,
    sepia_amount: f32,
    time_seconds: f32,
    gaussian_radius: f32,
    kuwahara_radius: f32,
    nightvision_gain: f32,
    bloom_tint_r: f32,
    bloom_tint_g: f32,
    bloom_tint_b: f32,
    grain_size: f32,
    denoise_strength: f32,
    enabled_bloom: u32,
    enabled_color_correct: u32,
    enabled_vignette: u32,
    enabled_chromatic_aberration: u32,
    enabled_grain: u32,
    enabled_dither: u32,
    enabled_sharpen: u32,
    enabled_pixelate: u32,
    enabled_scanlines: u32,
    enabled_edges: u32,
    enabled_distortion: u32,
    enabled_letterbox: u32,
    enabled_halation: u32,
    enabled_radial_blur: u32,
    enabled_hue_shift: u32,
    enabled_lift_gamma_gain: u32,
    enabled_posterize: u32,
    enabled_glitch: u32,
    enabled_zoom: u32,
    enabled_sepia: u32,
    animated_grain: u32,
    tonemap_mode: u32,
    enabled_gaussian_blur: u32,
    enabled_kuwahara: u32,
    enabled_nightvision: u32,
    flip_horizontal: u32,
    flip_vertical: u32,
    enabled_denoise: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
};

const PI: f32 = 3.14159265359;

// ====== Bloom Prefilter + Downsample ======
@group(0) @binding(0) var<storage, read> accum_buffer: array<vec4<f32>>;
@group(0) @binding(1) var bloom_a_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var<uniform> params: Params;
@group(0) @binding(3) var linear_sampler: sampler;

fn bloom_brightness(c: vec3<f32>) -> f32 {
    return max(c.r, max(c.g, c.b));
}

fn bloom_prefilter_color(col: vec3<f32>) -> vec3<f32> {
    let brightness = bloom_brightness(col);
    let knee = params.bloom_threshold * params.bloom_soft_threshold;
    var soft = brightness - params.bloom_threshold + knee;
    soft = clamp(soft, 0.0, 2.0 * knee);
    soft = soft * soft / (4.0 * knee + 0.00001);
    let contribution = max(soft, brightness - params.bloom_threshold);
    return col * (contribution / max(contribution, 0.00001));
}

@compute @workgroup_size(8, 8)
fn bloom_prefilter(@builtin(global_invocation_id) gid: vec3<u32>) {
    let half_dims = textureDimensions(bloom_a_out);
    let half_w = half_dims.x;
    let half_h = half_dims.y;
    if (gid.x >= half_w || gid.y >= half_h) { return; }

    let full_w = half_w * 2u;
    let full_h = half_h * 2u;

    let cx = i32(gid.x) * 2;
    let cy = i32(gid.y) * 2;
    let d = i32(params.bloom_downsample_delta);
    let fw = i32(full_w);
    let fh = i32(full_h);

    let x1 = clamp(cx - d, 0, fw - 1);
    let x2 = clamp(cx + d, 0, fw - 1);
    let y1 = clamp(cy - d, 0, fh - 1);
    let y2 = clamp(cy + d, 0, fh - 1);

    let s1 = accum_buffer[u32(y1) * full_w + u32(x1)];
    let s2 = accum_buffer[u32(y1) * full_w + u32(x2)];
    let s3 = accum_buffer[u32(y2) * full_w + u32(x1)];
    let s4 = accum_buffer[u32(y2) * full_w + u32(x2)];

    var color: vec3<f32>;
    if (params.bloom_use_karis == 1u) {
        let w1 = 1.0 / (bloom_brightness(s1.rgb) + params.bloom_luminance_bias);
        let w2 = 1.0 / (bloom_brightness(s2.rgb) + params.bloom_luminance_bias);
        let w3 = 1.0 / (bloom_brightness(s3.rgb) + params.bloom_luminance_bias);
        let w4 = 1.0 / (bloom_brightness(s4.rgb) + params.bloom_luminance_bias);
        let total = w1 + w2 + w3 + w4;
        color = (s1.rgb * w1 + s2.rgb * w2 + s3.rgb * w3 + s4.rgb * w4) / total;
    } else {
        color = (s1.rgb + s2.rgb + s3.rgb + s4.rgb) * 0.25;
    }

    color = bloom_prefilter_color(color);
    textureStore(bloom_a_out, vec2<i32>(gid.xy), vec4<f32>(color, 1.0));
}

// ====== Bloom Blur Horizontal ======
@group(0) @binding(0) var bloom_a_tex: texture_2d<f32>;
@group(0) @binding(1) var bloom_b_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var blur_sampler: sampler;

@compute @workgroup_size(8, 8)
fn bloom_blur_h(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(bloom_a_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }

    let uv = (vec2<f32>(gid.xy) + 0.5) / vec2<f32>(f32(dims.x), f32(dims.y));
    let texel = vec2<f32>(1.0 / f32(dims.x), 0.0);

    let s1 = textureSampleLevel(bloom_a_tex, blur_sampler, uv - texel * 1.3846153846, 0.0);
    let s2 = textureSampleLevel(bloom_a_tex, blur_sampler, uv, 0.0);
    let s3 = textureSampleLevel(bloom_a_tex, blur_sampler, uv + texel * 1.3846153846, 0.0);

    let color = s1.rgb * 0.319453 + s2.rgb * 0.361094 + s3.rgb * 0.319453;
    textureStore(bloom_b_out, vec2<i32>(gid.xy), vec4<f32>(color, 1.0));
}

// ====== Bloom Blur Vertical ======
@group(0) @binding(0) var bloom_b_tex: texture_2d<f32>;
@group(0) @binding(1) var bloom_a_v_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var blur_sampler_v: sampler;

@compute @workgroup_size(8, 8)
fn bloom_blur_v(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(bloom_b_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }

    let uv = (vec2<f32>(gid.xy) + 0.5) / vec2<f32>(f32(dims.x), f32(dims.y));
    let texel = vec2<f32>(0.0, 1.0 / f32(dims.y));

    let s1 = textureSampleLevel(bloom_b_tex, blur_sampler_v, uv - texel * 1.3846153846, 0.0);
    let s2 = textureSampleLevel(bloom_b_tex, blur_sampler_v, uv, 0.0);
    let s3 = textureSampleLevel(bloom_b_tex, blur_sampler_v, uv + texel * 1.3846153846, 0.0);

    let color = s1.rgb * 0.319453 + s2.rgb * 0.361094 + s3.rgb * 0.319453;
    textureStore(bloom_a_v_out, vec2<i32>(gid.xy), vec4<f32>(color, 1.0));
}

// ====== Composite Bindings ======
@group(0) @binding(0) var<storage, read> accum_read: array<vec4<f32>>;
@group(0) @binding(1) var bloom_composite_tex: texture_2d<f32>;
@group(0) @binding(2) var render_target: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<uniform> params_c: Params;
@group(0) @binding(4) var composite_sampler: sampler;

// ====== Helper Functions ======
fn pp_luminance(v: vec3<f32>) -> f32 {
    return max(0.00001, dot(v, vec3<f32>(0.2127, 0.7152, 0.0722)));
}

fn pp_white_balance(col: vec3<f32>, temp: f32, tint: f32) -> vec3<f32> {
    let t1 = temp * 10.0 / 6.0;
    let t2 = tint * 10.0 / 6.0;
    var wx = 0.31271 - t1 * select(0.1, 0.05, t1 < 0.0);
    let sy = 2.87 * wx - 3.0 * wx * wx - 0.27509507;
    let wy = sy + t2 * 0.05;

    let Y = 1.0;
    let Xn = Y * wx / wy;
    let Zn = Y * (1.0 - wx - wy) / wy;
    let L = 0.7328 * Xn + 0.4296 * Y - 0.1624 * Zn;
    let M = -0.7036 * Xn + 1.6975 * Y + 0.0061 * Zn;
    let S = 0.0030 * Xn + 0.0136 * Y + 0.9834 * Zn;

    let w1 = vec3<f32>(0.949237, 1.03542, 1.08728);
    let w2 = vec3<f32>(L, M, S);
    let balance = w1 / w2;

    let m1 = vec3<f32>(0.390405, 0.0708416, 0.0231082);
    let m2 = vec3<f32>(0.549941, 0.963172, 0.128021);
    let m3 = vec3<f32>(0.00892632, 0.00135775, 0.936245);
    let lms = vec3<f32>(dot(m1, col), dot(m2, col), dot(m3, col));

    let r1 = vec3<f32>(2.85847, -0.210182, -0.0418120);
    let r2 = vec3<f32>(-1.62879, 1.15820, -0.118169);
    let r3 = vec3<f32>(-0.0248910, 0.000324281, 1.06867);
    let balanced = lms * balance;
    return vec3<f32>(dot(r1, balanced), dot(r2, balanced), dot(r3, balanced));
}

fn apply_vignette(color: vec3<f32>, uv: vec2<f32>, p: Params) -> vec3<f32> {
    var pos = uv - 0.5;
    pos *= vec2<f32>(p.vignette_size_x, p.vignette_size_y);
    pos += 0.5;

    var d = abs(pos - (vec2<f32>(0.5) + vec2<f32>(p.vignette_offset_x, p.vignette_offset_y))) * p.vignette_intensity;
    d = pow(clamp(d, vec2<f32>(0.0), vec2<f32>(1.0)), vec2<f32>(p.vignette_roundness));
    let vf = pow(clamp(1.0 - dot(d, d), 0.0, 1.0), p.vignette_smoothness);

    return mix(vec3<f32>(p.vignette_color_r, p.vignette_color_g, p.vignette_color_b), color, vf);
}

fn pp_hash(n: u32) -> f32 {
    var m = n;
    m = (m << 13u) ^ m;
    m = m * (m * m * 15731u + 789221u) + 1376312589u;
    return f32(m & 0x7fffffffu) / f32(0x7fffffff);
}

var<private> BAYER4: array<f32, 16> = array<f32, 16>(
    0.0, 8.0, 2.0, 10.0,
    12.0, 4.0, 14.0, 6.0,
    3.0, 11.0, 1.0, 9.0,
    15.0, 7.0, 13.0, 5.0
);

fn bayer4_val(x: i32, y: i32) -> f32 {
    let mx = (x % 4 + 4) % 4;
    let my = (y % 4 + 4) % 4;
    return BAYER4[mx + my * 4] * (1.0 / 16.0) - 0.5;
}

fn aces_film(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn reinhard(x: vec3<f32>) -> vec3<f32> {
    return clamp(x / (vec3<f32>(1.0) + x), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn uncharted2_filmic(x: vec3<f32>) -> vec3<f32> {
    let a = 0.15;
    let b = 0.50;
    let c = 0.10;
    let d = 0.20;
    let e = 0.02;
    let f = 0.30;
    let w = 11.2;
    let curve = ((x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f)) - e / f;
    let white_scale = ((w * (a * w + c * b) + d * e) / (w * (a * w + b) + d * f)) - e / f;
    return clamp(curve / white_scale, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn hue_rotate(c: vec3<f32>, deg: f32) -> vec3<f32> {
    let a = radians(deg);
    let cs = cos(a);
    let sn = sin(a);
    let r_row = vec3<f32>(
        0.213 + cs * 0.787 - sn * 0.213,
        0.715 - cs * 0.715 - sn * 0.715,
        0.072 - cs * 0.072 + sn * 0.928);
    let g_row = vec3<f32>(
        0.213 - cs * 0.213 + sn * 0.143,
        0.715 + cs * 0.285 + sn * 0.140,
        0.072 - cs * 0.072 - sn * 0.283);
    let b_row = vec3<f32>(
        0.213 - cs * 0.213 - sn * 0.787,
        0.715 - cs * 0.715 + sn * 0.715,
        0.072 + cs * 0.928 + sn * 0.072);
    return vec3<f32>(dot(r_row, c), dot(g_row, c), dot(b_row, c));
}

fn apply_sepia(c: vec3<f32>, amount: f32) -> vec3<f32> {
    let sepia = vec3<f32>(
        dot(c, vec3<f32>(0.393, 0.769, 0.189)),
        dot(c, vec3<f32>(0.349, 0.686, 0.168)),
        dot(c, vec3<f32>(0.272, 0.534, 0.131)));
    return mix(c, clamp(sepia, vec3<f32>(0.0), vec3<f32>(1.0)), amount);
}

fn sample_accum(uv: vec2<f32>, dims: vec2<u32>) -> vec3<f32> {
    let px = clamp(vec2<i32>(uv * vec2<f32>(dims)), vec2<i32>(0), vec2<i32>(dims) - vec2<i32>(1));
    return accum_read[u32(px.y) * dims.x + u32(px.x)].rgb;
}

fn gaussian_blur_9tap(base_uv: vec2<f32>, dims: vec2<u32>, radius: f32) -> vec3<f32> {
    let texel = vec2<f32>(1.0 / f32(dims.x), 1.0 / f32(dims.y)) * max(radius, 0.5);
    let c = sample_accum(base_uv, dims);
    let e =
        sample_accum(base_uv + vec2<f32>(texel.x, 0.0), dims) +
        sample_accum(base_uv - vec2<f32>(texel.x, 0.0), dims) +
        sample_accum(base_uv + vec2<f32>(0.0, texel.y), dims) +
        sample_accum(base_uv - vec2<f32>(0.0, texel.y), dims);
    let k =
        sample_accum(base_uv + vec2<f32>( texel.x,  texel.y), dims) +
        sample_accum(base_uv + vec2<f32>( texel.x, -texel.y), dims) +
        sample_accum(base_uv + vec2<f32>(-texel.x,  texel.y), dims) +
        sample_accum(base_uv + vec2<f32>(-texel.x, -texel.y), dims);
    return (c * 4.0 + e * 2.0 + k) / 16.0;
}

fn kuwahara_filter(base_uv: vec2<f32>, dims: vec2<u32>, radius: i32) -> vec3<f32> {
    let texel = vec2<f32>(1.0 / f32(dims.x), 1.0 / f32(dims.y));
    var best_color = sample_accum(base_uv, dims);
    var best_var = 1.0e30;

    for (var q = 0; q < 4; q++) {
        let sx = select(-1.0, 1.0, q == 0 || q == 2);
        let sy = select(-1.0, 1.0, q <= 1);
        var sum = vec3<f32>(0.0);
        var sqsum = vec3<f32>(0.0);
        for (var dy = 0; dy <= radius; dy++) {
            for (var dx = 0; dx <= radius; dx++) {
                let off = vec2<f32>(f32(dx) * sx, f32(dy) * sy) * texel;
                let c = sample_accum(base_uv + off, dims);
                sum += c;
                sqsum += c * c;
            }
        }
        let count = f32((radius + 1) * (radius + 1));
        let mean = sum / count;
        let vari = dot(sqsum / count - mean * mean, vec3<f32>(1.0));
        if (vari < best_var) {
            best_var = vari;
            best_color = mean;
        }
    }
    return best_color;
}

fn apply_night_vision(c: vec3<f32>, gain: f32, seed: u32) -> vec3<f32> {
    let lum = pp_luminance(c);
    var nv = vec3<f32>(lum) * vec3<f32>(0.15, 1.0, 0.35) * gain;
    nv += (pp_hash(seed) - 0.5) * 0.12;
    return nv;
}

fn bilateral_denoise(center_px: vec2<i32>, dims: vec2<u32>, strength: f32) -> vec3<f32> {
    let c_idx = clamp(center_px, vec2<i32>(0), vec2<i32>(dims) - vec2<i32>(1));
    let center = accum_read[u32(c_idx.y) * dims.x + u32(c_idx.x)].rgb;
    let c_lum = pp_luminance(center);

    let sigma = mix(0.02, 0.5, clamp(strength, 0.0, 1.0));
    let inv_2s2 = 1.0 / (2.0 * sigma * sigma);

    var acc = vec3<f32>(0.0);
    var wsum = 0.0;
    for (var dy = -2; dy <= 2; dy++) {
        for (var dx = -2; dx <= 2; dx++) {
            let p = clamp(center_px + vec2<i32>(dx, dy), vec2<i32>(0), vec2<i32>(dims) - vec2<i32>(1));
            let s = accum_read[u32(p.y) * dims.x + u32(p.x)].rgb;
            let dl = pp_luminance(s) - c_lum;
            let spatial = exp(-f32(dx * dx + dy * dy) * 0.25);
            let range_w = exp(-dl * dl * inv_2s2);
            let w = spatial * range_w;
            acc += s * w;
            wsum += w;
        }
    }
    return acc / max(wsum, 0.0001);
}

fn get_warped_uv(uv: vec2<f32>, dims: vec2<u32>) -> vec2<f32> {
    var out_uv = uv;

    if (params_c.enabled_pixelate == 1u) {
        let size = max(params_c.pixelate_size, 1.0);
        let blocks = vec2<f32>(dims) / size;
        out_uv = (floor(out_uv * blocks) + 0.5) / blocks;
    }

    if (params_c.enabled_distortion == 1u) {
        var d = out_uv - 0.5;
        let r2 = dot(d, d);
        d = d * (1.0 + params_c.distortion_k1 * r2 + params_c.distortion_k2 * r2 * r2);
        out_uv = d + 0.5;
    }

    return out_uv;
}

fn accum_lum(px: vec2<i32>, dims: vec2<u32>) -> f32 {
    let c = clamp(px, vec2<i32>(0), vec2<i32>(dims) - vec2<i32>(1));
    return pp_luminance(accum_read[u32(c.y) * dims.x + u32(c.x)].rgb);
}

@compute @workgroup_size(16, 16)
fn postfx_composite(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(render_target);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }

    let uv = vec2<f32>(gid.xy) / vec2<f32>(f32(dims.x), f32(dims.y));

    // ===== Flip =====
    var uv_base = uv;
    if (params_c.flip_horizontal == 1u) { uv_base.x = 1.0 - uv_base.x; }
    if (params_c.flip_vertical == 1u) { uv_base.y = 1.0 - uv_base.y; }

    // ===== Zoom =====
    var sample_uv = uv_base;
    if (params_c.enabled_zoom == 1u && params_c.zoom_factor > 1.001) {
        sample_uv = (uv - 0.5) / params_c.zoom_factor + 0.5;
    }

    // ===== Glitch (VHS band displacement) =====
    if (params_c.enabled_glitch == 1u && params_c.glitch_intensity > 0.001) {
        let band = floor(sample_uv.y * f32(dims.y) / 4.0);
        let t = floor(params_c.time_seconds * max(params_c.glitch_speed, 0.001));
        let h1 = pp_hash(u32(band) * 7919u + u32(t) * 131u);
        let band_active = step(1.0 - clamp(params_c.glitch_intensity, 0.0, 1.0), h1);
        let h2 = pp_hash(u32(band) * 104729u + u32(t) * 17u);
        sample_uv.x = sample_uv.x + (h2 - 0.5) * 0.12 * params_c.glitch_intensity * band_active;
    }

    // ===== Pixelate + Lens Distortion =====
    let base_uv = get_warped_uv(sample_uv, dims);

    // ===== Base color (optionally radial blurred) =====
    var color: vec3<f32>;
    if (params_c.enabled_radial_blur == 1u && params_c.radial_intensity > 0.001) {
        let center = vec2<f32>(params_c.radial_center_x, params_c.radial_center_y);
        var acc = vec3<f32>(0.0);
        for (var i = 0u; i < 8u; i++) {
            let fi = f32(i) / 8.0;
            let suv = base_uv - (base_uv - center) * params_c.radial_intensity * fi;
            acc += sample_accum(suv, dims);
        }
        color = acc / 8.0;
    } else {
        color = sample_accum(base_uv, dims);
    }

    // ===== Denoise (edge-aware bilateral) =====
    if (params_c.enabled_denoise == 1u) {
        color = bilateral_denoise(vec2<i32>(gid.xy), dims, params_c.denoise_strength);
    }

    if (params_c.enabled_bloom == 1u) {
        let bloom_tint = vec3<f32>(params_c.bloom_tint_r, params_c.bloom_tint_g, params_c.bloom_tint_b);
        let bloom = clamp(textureSampleLevel(bloom_composite_tex, composite_sampler, base_uv, 0.0).rgb * params_c.bloom_intensity * bloom_tint, vec3<f32>(0.0), vec3<f32>(8.0));
        if (params_c.bloom_blend_mode == 1u) {
            let base = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
            color = vec3<f32>(1.0) - (vec3<f32>(1.0) - base) * (vec3<f32>(1.0) - clamp(bloom, vec3<f32>(0.0), vec3<f32>(1.0)));
        } else if (params_c.bloom_blend_mode == 2u) {
            color = color * (vec3<f32>(1.0) - clamp(bloom, vec3<f32>(0.0), vec3<f32>(1.0)));
        } else {
            color = color + bloom;
        }
    }

    // ===== Halation: tinted glow from bright pass =====
    if (params_c.enabled_halation == 1u && params_c.halation_intensity > 0.001) {
        let bloom = textureSampleLevel(bloom_composite_tex, composite_sampler, base_uv, 0.0).rgb;
        let halation_color = vec3<f32>(params_c.halation_color_r, params_c.halation_color_g, params_c.halation_color_b);
        color = color + bloom * halation_color * params_c.halation_intensity;
    }

    if (params_c.enabled_sharpen == 1u) {
        let texel = vec2<f32>(1.0 / f32(dims.x), 1.0 / f32(dims.y)) * max(params_c.sharpen_radius, 0.5);
        let center = sample_accum(base_uv, dims);
        let blur = (
            sample_accum(base_uv + vec2<f32>(texel.x, 0.0), dims) +
            sample_accum(base_uv - vec2<f32>(texel.x, 0.0), dims) +
            sample_accum(base_uv + vec2<f32>(0.0, texel.y), dims) +
            sample_accum(base_uv - vec2<f32>(0.0, texel.y), dims)
        ) * 0.25;
        color = center + (center - blur) * params_c.sharpen_amount;
    }

    if (params_c.enabled_edges == 1u) {
        let px = vec2<i32>(gid.xy);
        let tl = accum_lum(px + vec2<i32>(-1, -1), dims);
        let tc = accum_lum(px + vec2<i32>( 0, -1), dims);
        let tr = accum_lum(px + vec2<i32>( 1, -1), dims);
        let ml = accum_lum(px + vec2<i32>(-1,  0), dims);
        let mr = accum_lum(px + vec2<i32>( 1,  0), dims);
        let bl = accum_lum(px + vec2<i32>(-1,  1), dims);
        let bc = accum_lum(px + vec2<i32>( 0,  1), dims);
        let br = accum_lum(px + vec2<i32>( 1,  1), dims);

        let gx = (tr + 2.0 * mr + br) - (tl + 2.0 * ml + bl);
        let gy = (bl + 2.0 * bc + br) - (tl + 2.0 * tc + tr);
        let mag = sqrt(gx * gx + gy * gy);

        let mask = smoothstep(params_c.edge_threshold, params_c.edge_threshold * 2.0 + 0.001, mag) * params_c.edge_intensity;
        let edge_color = vec3<f32>(params_c.edge_color_r, params_c.edge_color_g, params_c.edge_color_b);
        color = mix(color, edge_color, clamp(mask, 0.0, 1.0));
    }

    if (params_c.enabled_color_correct == 1u) {
        color *= params_c.exposure;
        color = max(vec3<f32>(0.0), color);
        color = pp_white_balance(color, params_c.temperature, params_c.tint);
        color = max(vec3<f32>(0.0), color);
        color = params_c.contrast * (color - params_c.linear_midpoint) + params_c.linear_midpoint + params_c.brightness;
        color = max(vec3<f32>(0.0), color);
        color *= vec3<f32>(params_c.color_filter_r, params_c.color_filter_g, params_c.color_filter_b) * params_c.filter_intensity;
        color = mix(vec3<f32>(pp_luminance(color)), color, params_c.saturation);
    }

    // ===== Hue Shift =====
    if (params_c.enabled_hue_shift == 1u && abs(params_c.hue_shift_degrees) > 0.01) {
        color = hue_rotate(max(color, vec3<f32>(0.0)), params_c.hue_shift_degrees);
    }

    // ===== Lift / Gamma / Gain =====
    if (params_c.enabled_lift_gamma_gain == 1u) {
        let lift = vec3<f32>(params_c.lift_r, params_c.lift_g, params_c.lift_b);
        let gain = vec3<f32>(params_c.gain_r, params_c.gain_g, params_c.gain_b);
        color = max(color, vec3<f32>(0.0)) * gain + lift;
        color = pow(max(color, vec3<f32>(0.0)), vec3<f32>(1.0 / max(params_c.gamma_value, 0.05)));
    }

    if (params_c.enabled_chromatic_aberration == 1u) {
        var pos = base_uv - 0.5;
        pos -= vec2<f32>(params_c.ca_focal_offset_x, params_c.ca_focal_offset_y);
        pos *= vec2<f32>(params_c.ca_radius_x, params_c.ca_radius_y);
        pos += 0.5;

        let direction = pos - 0.5;
        let intensity = clamp(pow(abs(length(pos - 0.5)), params_c.ca_hardness), 0.0, 1.0) * params_c.ca_intensity;

        let rUV = base_uv + direction * params_c.ca_offset_r * intensity;
        let gUV = base_uv + direction * params_c.ca_offset_g * intensity;
        let bUV = base_uv + direction * params_c.ca_offset_b * intensity;

        color = vec3<f32>(
            sample_accum(rUV, dims).r,
            sample_accum(gUV, dims).g,
            sample_accum(bUV, dims).b
        );
    }

    // ===== Gaussian Blur =====
    if (params_c.enabled_gaussian_blur == 1u && params_c.gaussian_radius > 0.01) {
        color = gaussian_blur_9tap(base_uv, dims, params_c.gaussian_radius);
    }

    // ===== Kuwahara (painterly) =====
    if (params_c.enabled_kuwahara == 1u) {
        color = kuwahara_filter(base_uv, dims, i32(clamp(params_c.kuwahara_radius, 1.0, 4.0)));
    }

    // ===== Tone Mapping =====
    if (params_c.tonemap_mode == 0u) {
        color = aces_film(color * 0.85);
    } else if (params_c.tonemap_mode == 1u) {
        color = reinhard(color * 0.85);
    } else if (params_c.tonemap_mode == 2u) {
        color = uncharted2_filmic(color * 0.85);
    } else {
        color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    color = pow(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));

    // ===== Night Vision =====
    if (params_c.enabled_nightvision == 1u) {
        let nv_seed = gid.x * 7919u + gid.y * 104729u + u32(params_c.time_seconds * 60.0);
        var nv = apply_night_vision(color, params_c.nightvision_gain, nv_seed);
        let d = length(uv - 0.5);
        nv *= smoothstep(0.85, 0.35, d);
        color = nv;
    }

    // ===== Posterize =====
    if (params_c.enabled_posterize == 1u) {
        let levels = max(params_c.posterize_levels, 2.0);
        color.r = floor(color.r * (levels - 1.0) + 0.5) / (levels - 1.0);
        color.g = floor(color.g * (levels - 1.0) + 0.5) / (levels - 1.0);
        color.b = floor(color.b * (levels - 1.0) + 0.5) / (levels - 1.0);
    }

    if (params_c.enabled_scanlines == 1u) {
        let sl = 0.5 + 0.5 * sin(uv.y * f32(dims.y) * PI * params_c.scanline_frequency);
        color *= mix(1.0, sl, clamp(params_c.scanline_intensity, 0.0, 1.0));
    }

    if (params_c.enabled_vignette == 1u) {
        color = apply_vignette(color, uv, params_c);
    }

    if (params_c.enabled_grain == 1u) {
        let gs = max(params_c.grain_size, 1.0);
        let gx = u32(floor(f32(gid.x) / gs));
        let gy = u32(floor(f32(gid.y) / gs));
        var seed = gx + gy * dims.x + u32(params_c.grain_response * 1000.0);
        if (params_c.animated_grain == 1u) {
            seed = seed + u32(params_c.time_seconds * 60.0) * 9781u;
        }
        let noise = pp_hash(seed) * 2.0 - 1.0;
        let weight = 1.0 - sqrt(pp_luminance(color));
        color = color + color * noise * params_c.grain_intensity * mix(1.0, weight, params_c.grain_response);
    }

    if (params_c.enabled_sepia == 1u && params_c.sepia_amount > 0.001) {
        color = apply_sepia(color, params_c.sepia_amount);
    }

    if (params_c.enabled_dither == 1u) {
        let noise = bayer4_val(i32(gid.x), i32(gid.y));
        var dithered = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)) + params_c.dither_spread * noise;
        dithered.r = floor((params_c.dither_color_count_r - 1.0) * dithered.r + 0.5) / (params_c.dither_color_count_r - 1.0);
        dithered.g = floor((params_c.dither_color_count_g - 1.0) * dithered.g + 0.5) / (params_c.dither_color_count_g - 1.0);
        dithered.b = floor((params_c.dither_color_count_b - 1.0) * dithered.b + 0.5) / (params_c.dither_color_count_b - 1.0);
        color = clamp(dithered, vec3<f32>(0.0), vec3<f32>(1.0));
    }

    if (params_c.enabled_letterbox == 1u && params_c.letterbox_amount > 0.0) {
        let bar = min(params_c.letterbox_amount, 0.49);
        if (uv.y < bar || uv.y > 1.0 - bar) {
            color = vec3<f32>(0.0);
        }
    }

    textureStore(render_target, vec2<i32>(gid.xy), vec4<f32>(color, 1.0));
}
