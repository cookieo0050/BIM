use crate::editor::theme;
use crate::editor::{hierarchy, inspector};
use crate::scene::{GpuPrimitive, MaterialComponent, ShapeType, Scene, TransformComponent};
use bytemuck;
use eframe::egui_wgpu;
use egui::Context;
use glam::Vec3;

const MAX_ENTITY_TEXTURES: u32 = 16;
const ENTITY_TEX_SIZE: u32 = 512;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    pos: [f32; 3],
    _pad1: f32,
    dir: [f32; 3],
    _pad2: f32,
    up: [f32; 3],
    _pad3: f32,
    resolution: [f32; 2],
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
    sun_dir: [f32; 3],
    sun_angular_radius: f32,
    sun_color: [f32; 3],
    floor_enabled: u32,
    floor_color: [f32; 3],
    floor_roughness: f32,
    studio_top: [f32; 3],
    _pad4: f32,
    studio_bottom: [f32; 3],
    _pad5: f32,
    fog_color: [f32; 3],
    fog_enabled: u32,
    _pad6: [f32; 3],
    fog_density: f32,
    fill_dir: [f32; 3],
    fill_intensity: f32,
    fill_color: [f32; 3],
    fill_enabled: u32,
    skybox_rotation: f32,
    firefly_clamp: f32,
    floor_grid: u32,
    camera_roll: f32,
    ambient_color: [f32; 3],
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
    floor_grid_color: [f32; 3],
    _pad9: f32,
    floor_emissive: [f32; 3],
    _pad10: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SkyboxUniform {
    color: [f32; 3],
    mode: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PostFXUniform {
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
    dirblur_angle: f32,
    dirblur_distance: f32,
    tiltshift_focus: f32,
    tiltshift_range: f32,
    tiltshift_blur: f32,
    thermal_mix: f32,
    duotone_shadow_r: f32,
    duotone_shadow_g: f32,
    duotone_shadow_b: f32,
    duotone_highlight_r: f32,
    duotone_highlight_g: f32,
    duotone_highlight_b: f32,
    duotone_amount: f32,
    halftone_size: f32,
    halftone_mix: f32,
    warp_amplitude: f32,
    warp_frequency: f32,
    warp_speed: f32,
    film_scratches: f32,
    film_flicker: f32,
    flare_intensity: f32,
    flare_ghosts: f32,
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
    enabled_dir_blur: u32,
    enabled_tilt_shift: u32,
    enabled_thermal: u32,
    enabled_duotone: u32,
    enabled_halftone: u32,
    enabled_water_warp: u32,
    enabled_old_film: u32,
    enabled_flare: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

struct PostFXState {
    enabled_bloom: bool,
    bloom_threshold: f32,
    bloom_soft_threshold: f32,
    bloom_intensity: f32,
    bloom_use_karis: bool,
    bloom_luminance_bias: f32,
    bloom_downsample_delta: f32,
    bloom_upsample_delta: f32,
    bloom_blend_mode: u32,
    bloom_tint: [f32; 3],

    enabled_color_correct: bool,
    exposure: f32,
    temperature: f32,
    tint: f32,
    contrast: f32,
    linear_midpoint: f32,
    brightness: f32,
    filter_intensity: f32,
    saturation: f32,
    color_filter: [f32; 3],

    enabled_vignette: bool,
    vignette_intensity: f32,
    vignette_roundness: f32,
    vignette_smoothness: f32,
    vignette_color: [f32; 3],
    vignette_size: [f32; 2],
    vignette_offset: [f32; 2],

    enabled_chromatic_aberration: bool,
    ca_intensity: f32,
    ca_hardness: f32,
    ca_color_offsets: [f32; 3],
    ca_focal_offset: [f32; 2],
    ca_radius: [f32; 2],

    enabled_grain: bool,
    grain_intensity: f32,
    grain_response: f32,
    grain_size: f32,

    enabled_denoise: bool,
    denoise_strength: f32,

    enabled_dir_blur: bool,
    dirblur_angle: f32,
    dirblur_distance: f32,

    enabled_tilt_shift: bool,
    tiltshift_focus: f32,
    tiltshift_range: f32,
    tiltshift_blur: f32,

    enabled_flare: bool,
    flare_intensity: f32,
    flare_ghosts: i32,

    enabled_water_warp: bool,
    warp_amplitude: f32,
    warp_frequency: f32,
    warp_speed: f32,

    enabled_thermal: bool,
    thermal_mix: f32,

    enabled_duotone: bool,
    duotone_shadow: [f32; 3],
    duotone_highlight: [f32; 3],
    duotone_amount: f32,

    enabled_halftone: bool,
    halftone_size: f32,
    halftone_mix: f32,

    enabled_old_film: bool,
    film_scratches: f32,
    film_flicker: f32,

    enabled_dither: bool,
    dither_spread: f32,
    dither_color_counts: [f32; 3],

    enabled_sharpen: bool,
    sharpen_amount: f32,
    sharpen_radius: f32,

    enabled_pixelate: bool,
    pixelate_size: f32,

    enabled_scanlines: bool,
    scanline_intensity: f32,
    scanline_frequency: f32,

    enabled_edges: bool,
    edge_intensity: f32,
    edge_threshold: f32,
    edge_color: [f32; 3],

    enabled_distortion: bool,
    distortion_k1: f32,
    distortion_k2: f32,

    enabled_letterbox: bool,
    letterbox_amount: f32,

    enabled_halation: bool,
    halation_intensity: f32,
    halation_color: [f32; 3],

    enabled_radial_blur: bool,
    radial_intensity: f32,
    radial_center: [f32; 2],

    enabled_hue_shift: bool,
    hue_shift_degrees: f32,

    enabled_lift_gamma_gain: bool,
    lift: [f32; 3],
    gain: [f32; 3],
    gamma_value: f32,

    enabled_posterize: bool,
    posterize_levels: f32,

    enabled_glitch: bool,
    glitch_intensity: f32,
    glitch_speed: f32,

    enabled_zoom: bool,
    zoom_factor: f32,

    enabled_sepia: bool,
    sepia_amount: f32,

    enabled_gaussian_blur: bool,
    gaussian_radius: f32,

    enabled_kuwahara: bool,
    kuwahara_radius: f32,

    enabled_nightvision: bool,
    nightvision_gain: f32,

    flip_horizontal: bool,
    flip_vertical: bool,

    animated_grain: bool,
    tonemap_mode: i32,
}

impl Default for PostFXState {
    fn default() -> Self {
        Self {
            enabled_bloom: false,
            bloom_threshold: 0.9,
            bloom_soft_threshold: 0.5,
            bloom_intensity: 0.8,
            bloom_use_karis: true,
            bloom_luminance_bias: 0.001,
            bloom_downsample_delta: 1.0,
            bloom_upsample_delta: 1.0,
            bloom_blend_mode: 0,
            bloom_tint: [1.0, 1.0, 1.0],

            enabled_color_correct: false,
            exposure: 1.0,
            temperature: 0.0,
            tint: 0.0,
            contrast: 1.0,
            linear_midpoint: 0.5,
            brightness: 0.0,
            filter_intensity: 1.0,
            saturation: 1.0,
            color_filter: [1.0, 1.0, 1.0],

            enabled_vignette: false,
            vignette_intensity: 1.0,
            vignette_roundness: 1.0,
            vignette_smoothness: 1.0,
            vignette_color: [0.0, 0.0, 0.0],
            vignette_size: [1.0, 1.0],
            vignette_offset: [0.0, 0.0],

            enabled_chromatic_aberration: false,
            ca_intensity: 0.0,
            ca_hardness: 1.0,
            ca_color_offsets: [0.0, 0.0, 0.0],
            ca_focal_offset: [0.0, 0.0],
            ca_radius: [1.0, 1.0],

            enabled_grain: false,
            grain_intensity: 0.15,
            grain_response: 0.15,
            grain_size: 1.0,

            enabled_denoise: false,
            denoise_strength: 0.3,

            enabled_dir_blur: false,
            dirblur_angle: 0.0,
            dirblur_distance: 0.3,

            enabled_tilt_shift: false,
            tiltshift_focus: 0.5,
            tiltshift_range: 0.15,
            tiltshift_blur: 0.4,

            enabled_flare: false,
            flare_intensity: 0.6,
            flare_ghosts: 4,

            enabled_water_warp: false,
            warp_amplitude: 0.4,
            warp_frequency: 6.0,
            warp_speed: 2.0,

            enabled_thermal: false,
            thermal_mix: 1.0,

            enabled_duotone: false,
            duotone_shadow: [0.1, 0.1, 0.35],
            duotone_highlight: [1.0, 0.9, 0.75],
            duotone_amount: 1.0,

            enabled_halftone: false,
            halftone_size: 6.0,
            halftone_mix: 1.0,

            enabled_old_film: false,
            film_scratches: 0.4,
            film_flicker: 0.5,

            enabled_dither: false,
            dither_spread: 0.0625,
            dither_color_counts: [255.0, 255.0, 255.0],

            enabled_sharpen: false,
            sharpen_amount: 0.5,
            sharpen_radius: 1.0,

            enabled_pixelate: false,
            pixelate_size: 8.0,

            enabled_scanlines: false,
            scanline_intensity: 0.25,
            scanline_frequency: 1.0,

            enabled_edges: false,
            edge_intensity: 1.0,
            edge_threshold: 0.2,
            edge_color: [0.0, 0.0, 0.0],

            enabled_distortion: false,
            distortion_k1: 0.1,
            distortion_k2: 0.1,

            enabled_letterbox: false,
            letterbox_amount: 0.12,

            enabled_halation: false,
            halation_intensity: 0.5,
            halation_color: [1.0, 0.3, 0.15],

            enabled_radial_blur: false,
            radial_intensity: 0.35,
            radial_center: [0.5, 0.5],

            enabled_hue_shift: false,
            hue_shift_degrees: 30.0,

            enabled_lift_gamma_gain: false,
            lift: [0.0, 0.0, 0.0],
            gain: [1.0, 1.0, 1.0],
            gamma_value: 1.0,

            enabled_posterize: false,
            posterize_levels: 6.0,

            enabled_glitch: false,
            glitch_intensity: 0.3,
            glitch_speed: 8.0,

            enabled_zoom: false,
            zoom_factor: 1.5,

            enabled_sepia: false,
            sepia_amount: 0.8,

            enabled_gaussian_blur: false,
            gaussian_radius: 2.0,

            enabled_kuwahara: false,
            kuwahara_radius: 2.0,

            enabled_nightvision: false,
            nightvision_gain: 6.0,

            flip_horizontal: false,
            flip_vertical: false,

            animated_grain: false,
            tonemap_mode: 0,
        }
    }
}

impl PostFXState {
    fn to_uniform(&self, time_seconds: f32) -> PostFXUniform {
        PostFXUniform {
            bloom_threshold: self.bloom_threshold,
            bloom_soft_threshold: self.bloom_soft_threshold,
            bloom_intensity: self.bloom_intensity,
            bloom_use_karis: self.bloom_use_karis as u32,
            bloom_luminance_bias: self.bloom_luminance_bias,
            bloom_downsample_delta: self.bloom_downsample_delta,
            bloom_upsample_delta: self.bloom_upsample_delta,
            bloom_blend_mode: self.bloom_blend_mode,
            bloom_tint_r: self.bloom_tint[0],
            bloom_tint_g: self.bloom_tint[1],
            bloom_tint_b: self.bloom_tint[2],
            exposure: self.exposure,
            temperature: self.temperature,
            tint: self.tint,
            contrast: self.contrast,
            linear_midpoint: self.linear_midpoint,
            brightness: self.brightness,
            filter_intensity: self.filter_intensity,
            saturation: self.saturation,
            color_filter_r: self.color_filter[0],
            color_filter_g: self.color_filter[1],
            color_filter_b: self.color_filter[2],
            vignette_intensity: self.vignette_intensity,
            vignette_roundness: self.vignette_roundness,
            vignette_smoothness: self.vignette_smoothness,
            vignette_color_r: self.vignette_color[0],
            vignette_color_g: self.vignette_color[1],
            vignette_color_b: self.vignette_color[2],
            vignette_size_x: self.vignette_size[0],
            vignette_size_y: self.vignette_size[1],
            vignette_offset_x: self.vignette_offset[0],
            vignette_offset_y: self.vignette_offset[1],
            ca_intensity: self.ca_intensity,
            ca_hardness: self.ca_hardness,
            ca_offset_r: self.ca_color_offsets[0],
            ca_offset_g: self.ca_color_offsets[1],
            ca_offset_b: self.ca_color_offsets[2],
            ca_focal_offset_x: self.ca_focal_offset[0],
            ca_focal_offset_y: self.ca_focal_offset[1],
            ca_radius_x: self.ca_radius[0],
            ca_radius_y: self.ca_radius[1],
            grain_intensity: self.grain_intensity,
            grain_response: self.grain_response,
            grain_size: self.grain_size,
            denoise_strength: self.denoise_strength,
            dirblur_angle: self.dirblur_angle,
            dirblur_distance: self.dirblur_distance,
            tiltshift_focus: self.tiltshift_focus,
            tiltshift_range: self.tiltshift_range,
            tiltshift_blur: self.tiltshift_blur,
            thermal_mix: self.thermal_mix,
            duotone_shadow_r: self.duotone_shadow[0],
            duotone_shadow_g: self.duotone_shadow[1],
            duotone_shadow_b: self.duotone_shadow[2],
            duotone_highlight_r: self.duotone_highlight[0],
            duotone_highlight_g: self.duotone_highlight[1],
            duotone_highlight_b: self.duotone_highlight[2],
            duotone_amount: self.duotone_amount,
            halftone_size: self.halftone_size,
            halftone_mix: self.halftone_mix,
            warp_amplitude: self.warp_amplitude,
            warp_frequency: self.warp_frequency,
            warp_speed: self.warp_speed,
            film_scratches: self.film_scratches,
            film_flicker: self.film_flicker,
            flare_intensity: self.flare_intensity,
            flare_ghosts: self.flare_ghosts as f32,
            dither_spread: self.dither_spread,
            dither_color_count_r: self.dither_color_counts[0],
            dither_color_count_g: self.dither_color_counts[1],
            dither_color_count_b: self.dither_color_counts[2],
            sharpen_amount: self.sharpen_amount,
            sharpen_radius: self.sharpen_radius,
            pixelate_size: self.pixelate_size,
            scanline_intensity: self.scanline_intensity,
            scanline_frequency: self.scanline_frequency,
            edge_intensity: self.edge_intensity,
            edge_threshold: self.edge_threshold,
            edge_color_r: self.edge_color[0],
            edge_color_g: self.edge_color[1],
            edge_color_b: self.edge_color[2],
            distortion_k1: self.distortion_k1,
            distortion_k2: self.distortion_k2,
            letterbox_amount: self.letterbox_amount,
            halation_intensity: self.halation_intensity,
            halation_color_r: self.halation_color[0],
            halation_color_g: self.halation_color[1],
            halation_color_b: self.halation_color[2],
            radial_intensity: self.radial_intensity,
            radial_center_x: self.radial_center[0],
            radial_center_y: self.radial_center[1],
            hue_shift_degrees: self.hue_shift_degrees,
            lift_r: self.lift[0],
            lift_g: self.lift[1],
            lift_b: self.lift[2],
            gain_r: self.gain[0],
            gain_g: self.gain[1],
            gain_b: self.gain[2],
            gamma_value: self.gamma_value,
            posterize_levels: self.posterize_levels,
            glitch_intensity: self.glitch_intensity,
            glitch_speed: self.glitch_speed,
            zoom_factor: self.zoom_factor,
            sepia_amount: self.sepia_amount,
            time_seconds,
            gaussian_radius: self.gaussian_radius,
            kuwahara_radius: self.kuwahara_radius,
            nightvision_gain: self.nightvision_gain,
            enabled_bloom: self.enabled_bloom as u32,
            enabled_color_correct: self.enabled_color_correct as u32,
            enabled_vignette: self.enabled_vignette as u32,
            enabled_chromatic_aberration: self.enabled_chromatic_aberration as u32,
            enabled_grain: self.enabled_grain as u32,
            enabled_dither: self.enabled_dither as u32,
            enabled_sharpen: self.enabled_sharpen as u32,
            enabled_pixelate: self.enabled_pixelate as u32,
            enabled_scanlines: self.enabled_scanlines as u32,
            enabled_edges: self.enabled_edges as u32,
            enabled_distortion: self.enabled_distortion as u32,
            enabled_letterbox: self.enabled_letterbox as u32,
            enabled_halation: self.enabled_halation as u32,
            enabled_radial_blur: self.enabled_radial_blur as u32,
            enabled_hue_shift: self.enabled_hue_shift as u32,
            enabled_lift_gamma_gain: self.enabled_lift_gamma_gain as u32,
            enabled_posterize: self.enabled_posterize as u32,
            enabled_glitch: self.enabled_glitch as u32,
            enabled_zoom: self.enabled_zoom as u32,
            enabled_sepia: self.enabled_sepia as u32,
            animated_grain: self.animated_grain as u32,
            tonemap_mode: self.tonemap_mode as u32,
            enabled_gaussian_blur: self.enabled_gaussian_blur as u32,
            enabled_kuwahara: self.enabled_kuwahara as u32,
            enabled_nightvision: self.enabled_nightvision as u32,
            flip_horizontal: self.flip_horizontal as u32,
            flip_vertical: self.flip_vertical as u32,
            enabled_denoise: self.enabled_denoise as u32,
            enabled_dir_blur: self.enabled_dir_blur as u32,
            enabled_tilt_shift: self.enabled_tilt_shift as u32,
            enabled_thermal: self.enabled_thermal as u32,
            enabled_duotone: self.enabled_duotone as u32,
            enabled_halftone: self.enabled_halftone as u32,
            enabled_water_warp: self.enabled_water_warp as u32,
            enabled_old_film: self.enabled_old_film as u32,
            enabled_flare: self.enabled_flare as u32,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
            _pad3: 0,
        }
    }
}

fn lcg_next(state: &mut u64) -> f32 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*state >> 33) % 1_000_000) as f32 / 1_000_000.0
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SceneDto {
    entities: Vec<EntityDto>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EntityDto {
    name: String,
    shape: String,
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
    material: MaterialDto,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MaterialDto {
    albedo: [f32; 3],
    emissive: [f32; 3],
    roughness: f32,
    metallic: f32,
    ior: f32,
    opacity: f32,
    visible: bool,
    cast_shadow: bool,
    two_sided: bool,
    texture_id: u32,
    clearcoat: f32,
    sheen: f32,
    transmission: f32,
    emissive_intensity: f32,
    specular_tint: f32,
    uv_scale: [f32; 2],
    uv_offset: [f32; 2],
}

fn shape_to_string(s: &ShapeType) -> &'static str {
    match s {
        ShapeType::Sphere => "Sphere",
        ShapeType::Cube => "Cube",
        ShapeType::Cylinder => "Cylinder",
        ShapeType::Plane => "Plane",
    }
}

fn shape_from_string(s: &str) -> ShapeType {
    match s {
        "Cube" => ShapeType::Cube,
        "Cylinder" => ShapeType::Cylinder,
        "Plane" => ShapeType::Plane,
        _ => ShapeType::Sphere,
    }
}

pub struct SceneEditor {
    pub scene: Scene,
    pub selected_entity_index: Option<usize>,
    active_tab: usize,
    render_target: Option<egui::TextureId>,
    render_target_texture: Option<wgpu::Texture>,
    accumulation_buffer: Option<wgpu::Buffer>,
    viewport_width: u32,
    viewport_height: u32,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    camera_buffer: wgpu::Buffer,
    primitive_buffer: wgpu::Buffer,
    camera: CameraState,
    frame_index: u32,
    prev_camera_pos: Vec3,
    prev_camera_yaw: f32,
    prev_camera_pitch: f32,
    skybox_buffer: wgpu::Buffer,
    skybox_state: SkyboxState,
    env_texture: wgpu::Texture,
    env_texture_view: wgpu::TextureView,
    env_sampler: wgpu::Sampler,
    env_egui_id: Option<egui::TextureId>,
    raytracing_enabled: bool,
    sun_state: SunState,
    fill_state: FillLightState,
    ambient_state: AmbientState,
    floor_state: FloorState,
    quality_state: QualityState,
    fog_state: FogState,
    render_scale: f32,
    postfx_time: f32,
    entity_texture_array: Option<wgpu::Texture>,
    entity_texture_view: Option<wgpu::TextureView>,
    entity_sampler: wgpu::Sampler,
    entity_texture_used: Vec<bool>,
    postfx_state: PostFXState,
    postfx_buffer: wgpu::Buffer,
    postfx_sampler: wgpu::Sampler,
    bloom_a: Option<wgpu::Texture>,
    bloom_a_view: Option<wgpu::TextureView>,
    bloom_b: Option<wgpu::Texture>,
    bloom_b_view: Option<wgpu::TextureView>,
    bloom_prefilter_pipeline: wgpu::ComputePipeline,
    bloom_blur_h_pipeline: wgpu::ComputePipeline,
    bloom_blur_v_pipeline: wgpu::ComputePipeline,
    postfx_composite_pipeline: wgpu::ComputePipeline,
    bloom_prefilter_bg_layout: wgpu::BindGroupLayout,
    bloom_blur_h_bg_layout: wgpu::BindGroupLayout,
    bloom_blur_v_bg_layout: wgpu::BindGroupLayout,
    postfx_composite_bg_layout: wgpu::BindGroupLayout,
    bloom_prefilter_bg: Option<wgpu::BindGroup>,
    bloom_blur_h_bg: Option<wgpu::BindGroup>,
    bloom_blur_v_bg: Option<wgpu::BindGroup>,
    postfx_composite_bg: Option<wgpu::BindGroup>,
}

struct SkyboxState {
    mode: u32,
    color: [f32; 3],
    studio_top: [f32; 3],
    studio_bottom: [f32; 3],
    rotation_degrees: f32,
    sky_intensity: f32,
}

impl Default for SkyboxState {
    fn default() -> Self {
        Self {
            mode: 0,
            color: [0.5, 0.5, 0.5],
            studio_top: [0.92, 0.95, 1.0],
            studio_bottom: [0.75, 0.78, 0.82],
            rotation_degrees: 0.0,
            sky_intensity: 1.0,
        }
    }
}

struct FogState {
    enabled: bool,
    density: f32,
    color: [f32; 3],
}

impl Default for FogState {
    fn default() -> Self {
        Self {
            enabled: false,
            density: 0.04,
            color: [0.75, 0.78, 0.82],
        }
    }
}

struct CameraState {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
    sensitivity: f32,
    fov: f32,
    dof_enabled: bool,
    aperture: f32,
    focus_distance: f32,
    roll: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 1.0, -5.0),
            yaw: 0.0,
            pitch: 0.0,
            speed: 3.0,
            sensitivity: 0.003,
            fov: 38.0,
            dof_enabled: false,
            aperture: 0.1,
            focus_distance: 8.0,
            roll: 0.0,
        }
    }
}

struct SunState {
    enabled: bool,
    azimuth: f32,
    elevation: f32,
    intensity: f32,
    angular_radius_deg: f32,
    color: [f32; 3],
    shadows: bool,
}

impl Default for SunState {
    fn default() -> Self {
        Self {
            enabled: false,
            azimuth: 130.0,
            elevation: 45.0,
            intensity: 3.0,
            angular_radius_deg: 2.5,
            color: [1.0, 0.95, 0.88],
            shadows: true,
        }
    }
}

impl SunState {
    fn direction(&self) -> [f32; 3] {
        let az = self.azimuth.to_radians();
        let el = self.elevation.to_radians();
        let dir = Vec3::new(el.cos() * az.cos(), el.sin(), el.cos() * az.sin());
        dir.normalize().into()
    }
}

struct FillLightState {
    enabled: bool,
    azimuth: f32,
    elevation: f32,
    intensity: f32,
    color: [f32; 3],
}

impl Default for FillLightState {
    fn default() -> Self {
        Self {
            enabled: false,
            azimuth: 310.0,
            elevation: 25.0,
            intensity: 0.8,
            color: [0.7, 0.8, 1.0],
        }
    }
}

impl FillLightState {
    fn direction(&self) -> [f32; 3] {
        let az = self.azimuth.to_radians();
        let el = self.elevation.to_radians();
        let dir = Vec3::new(el.cos() * az.cos(), el.sin(), el.cos() * az.sin());
        dir.normalize().into()
    }
}

struct FloorState {
    enabled: bool,
    color: [f32; 3],
    roughness: f32,
    metallic: f32,
    ior: f32,
    grid: bool,
    grid_scale: f32,
    grid_thickness: f32,
    grid_color: [f32; 3],
    checker: bool,
    uv_scale: f32,
    emissive: [f32; 3],
    emissive_intensity: f32,
    height: f32,
}

impl Default for FloorState {
    fn default() -> Self {
        Self {
            enabled: true,
            color: [0.82, 0.83, 0.85],
            roughness: 0.15,
            metallic: 0.0,
            ior: 1.5,
            grid: true,
            grid_scale: 1.0,
            grid_thickness: 0.06,
            grid_color: [0.66, 0.68, 0.72],
            checker: false,
            uv_scale: 1.0,
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
            height: -1.0,
        }
    }
}

struct AmbientState {
    intensity: f32,
    color: [f32; 3],
}

impl Default for AmbientState {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            color: [1.0, 1.0, 1.0],
        }
    }
}

struct QualityState {
    max_bounces: u32,
    samples_per_pixel: u32,
    firefly_clamp: f32,
}

impl Default for QualityState {
    fn default() -> Self {
        Self {
            max_bounces: 3,
            samples_per_pixel: 1,
            firefly_clamp: 10.0,
        }
    }
}

impl CameraState {
    fn direction(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        Vec3::new(cos_pitch * sin_yaw, sin_pitch, cos_pitch * cos_yaw).normalize()
    }

    fn up(&self) -> Vec3 {
        let forward = self.direction();
        let right = Vec3::Y.cross(forward).normalize();
        let up = forward.cross(right).normalize();
        let (s, c) = self.roll.to_radians().sin_cos();
        (right * s + up * c).normalize()
    }

    fn update(&mut self, ctx: &Context) {
        let dt = ctx.input(|i| i.predicted_dt);

        let input = ctx.input(|i| {
            let keys: Vec<_> = i.keys_down.iter().copied().collect();
            let mouse_delta = i.pointer.delta();
            let right_held = i.pointer.button_down(egui::PointerButton::Secondary);
            (keys, mouse_delta, right_held)
        });

        let (keys, mouse_delta, right_held) = input;

        if right_held {
            self.yaw += mouse_delta.x * self.sensitivity;
            self.pitch -= mouse_delta.y * self.sensitivity;
            self.pitch = self.pitch.clamp(-1.5, 1.5);
        }

        let forward = self.direction();
        let right = Vec3::Y.cross(forward).normalize();
        let up = Vec3::Y;

        let mut movement = Vec3::ZERO;

        for key in &keys {
            match key {
                egui::Key::W => movement += forward,
                egui::Key::S => movement -= forward,
                egui::Key::A => movement -= right,
                egui::Key::D => movement += right,
                egui::Key::Space => movement += up,
                egui::Key::C => movement -= up,
                _ => {}
            }
        }

        if movement.length_squared() > 0.0 {
            movement = movement.normalize() * self.speed * dt;
            self.position += movement;
        }
    }
}

impl SceneEditor {
    pub fn new(render_state: &egui_wgpu::RenderState) -> Self {
        let device = &render_state.device;

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Raytracer Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(std::mem::size_of::<GpuPrimitive>() as u64)
                                .unwrap(),
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Raytracer Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "main",
            });

        let postfx_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PostFX Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../postprocess.wgsl").into()),
        });

        let bloom_prefilter_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom Prefilter BG Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bloom_blur_h_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom Blur H BG Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bloom_blur_v_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom Blur V BG Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let postfx_composite_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PostFX Composite BG Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bloom_prefilter_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bloom Prefilter Pipeline Layout"),
            bind_group_layouts: &[&bloom_prefilter_bg_layout],
            push_constant_ranges: &[],
        });

        let bloom_prefilter_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Bloom Prefilter Pipeline"),
            layout: Some(&bloom_prefilter_pipeline_layout),
            module: &postfx_shader,
            entry_point: "bloom_prefilter",
        });

        let bloom_blur_h_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bloom Blur H Pipeline Layout"),
            bind_group_layouts: &[&bloom_blur_h_bg_layout],
            push_constant_ranges: &[],
        });

        let bloom_blur_h_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Bloom Blur H Pipeline"),
            layout: Some(&bloom_blur_h_pipeline_layout),
            module: &postfx_shader,
            entry_point: "bloom_blur_h",
        });

        let bloom_blur_v_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bloom Blur V Pipeline Layout"),
            bind_group_layouts: &[&bloom_blur_v_bg_layout],
            push_constant_ranges: &[],
        });

        let bloom_blur_v_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Bloom Blur V Pipeline"),
            layout: Some(&bloom_blur_v_pipeline_layout),
            module: &postfx_shader,
            entry_point: "bloom_blur_v",
        });

        let postfx_composite_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PostFX Composite Pipeline Layout"),
            bind_group_layouts: &[&postfx_composite_bg_layout],
            push_constant_ranges: &[],
        });

        let postfx_composite_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("PostFX Composite Pipeline"),
            layout: Some(&postfx_composite_pipeline_layout),
            module: &postfx_shader,
            entry_point: "postfx_composite",
        });

        let postfx_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PostFX Uniform Buffer"),
            size: std::mem::size_of::<PostFXUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let postfx_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("PostFX Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let primitive_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Primitive Storage Buffer"),
            size: (std::mem::size_of::<GpuPrimitive>() * 256) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let skybox_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skybox Uniform Buffer"),
            size: std::mem::size_of::<SkyboxUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let env_texture = Self::create_default_env_texture(device, &render_state.queue);
        let env_texture_view =
            env_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let env_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Env Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let entity_texture_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Entity Texture Array"),
            size: wgpu::Extent3d {
                width: ENTITY_TEX_SIZE,
                height: ENTITY_TEX_SIZE,
                depth_or_array_layers: MAX_ENTITY_TEXTURES,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let default_pixel: [u8; 4] = [255, 255, 255, 255];
        for layer in 0..MAX_ENTITY_TEXTURES {
            let data: Vec<u8> = default_pixel.iter().copied().cycle().take((ENTITY_TEX_SIZE * ENTITY_TEX_SIZE * 4) as usize).collect();
            render_state.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &entity_texture_array,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: layer },
                    aspect: wgpu::TextureAspect::All,
                },
                &data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * ENTITY_TEX_SIZE),
                    rows_per_image: Some(ENTITY_TEX_SIZE),
                },
                wgpu::Extent3d {
                    width: ENTITY_TEX_SIZE,
                    height: ENTITY_TEX_SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }

        let entity_texture_view = entity_texture_array.create_view(&wgpu::TextureViewDescriptor::default());

        let entity_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Entity Texture Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let mut editor = Self {
            scene: Scene::new(),
            selected_entity_index: None,
            render_target: None,
            render_target_texture: None,
            accumulation_buffer: None,
            viewport_width: 1280,
            viewport_height: 720,
            compute_pipeline,
            bind_group_layout,
            bind_group: None,
            camera_buffer,
            primitive_buffer,
            camera: CameraState::default(),
            active_tab: 0,
            frame_index: 0,
            prev_camera_pos: Vec3::new(0.0, 1.0, -5.0),
            prev_camera_yaw: 0.0,
            prev_camera_pitch: 0.0,
            skybox_buffer,
            skybox_state: SkyboxState::default(),
            env_texture,
            env_texture_view,
            env_sampler,
            env_egui_id: None,
            raytracing_enabled: true,
            sun_state: SunState::default(),
            fill_state: FillLightState::default(),
            ambient_state: AmbientState::default(),
            floor_state: FloorState::default(),
            quality_state: QualityState::default(),
            fog_state: FogState::default(),
            render_scale: 1.0,
            postfx_time: 0.0,
            entity_texture_array: Some(entity_texture_array),
            entity_texture_view: Some(entity_texture_view),
            entity_sampler,
            entity_texture_used: vec![false; MAX_ENTITY_TEXTURES as usize],
            postfx_state: PostFXState::default(),
            postfx_buffer,
            postfx_sampler,
            bloom_a: None,
            bloom_a_view: None,
            bloom_b: None,
            bloom_b_view: None,
            bloom_prefilter_pipeline,
            bloom_blur_h_pipeline,
            bloom_blur_v_pipeline,
            postfx_composite_pipeline,
            bloom_prefilter_bg_layout,
            bloom_blur_h_bg_layout,
            bloom_blur_v_bg_layout,
            postfx_composite_bg_layout,
            bloom_prefilter_bg: None,
            bloom_blur_h_bg: None,
            bloom_blur_v_bg: None,
            postfx_composite_bg: None,
        };

        editor.create_viewport_textures(render_state);
        editor.create_bloom_textures(render_state);
        editor
    }

    fn create_default_env_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default Env Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let data: [u8; 4] = [255, 255, 255, 255];
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        tex
    }

    fn upload_env_texture(&mut self, render_state: &egui_wgpu::RenderState, rgba: &[u8], width: u32, height: u32) {
        let device = &render_state.device;
        let queue = &render_state.queue;

        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Environment Map"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.env_texture = tex;
        self.env_texture_view = self.env_texture.create_view(&wgpu::TextureViewDescriptor::default());

        if let Some(old_id) = self.env_egui_id {
            render_state.renderer.write().free_texture(&old_id);
        }
        let view = self.env_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let id = render_state.renderer.write().register_native_texture(
            device,
            &view,
            wgpu::FilterMode::Linear,
        );
        self.env_egui_id = Some(id);

        self.rebuild_bind_group(device);
        self.frame_index = 0;
    }

    fn rebuild_bind_group(&mut self, device: &wgpu::Device) {
        let accum_buf = self.accumulation_buffer.as_ref().unwrap();

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: accum_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.primitive_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.skybox_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.env_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.env_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(self.entity_texture_view.as_ref().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&self.entity_sampler),
                },
            ],
        }));
    }

    fn create_viewport_textures(&mut self, render_state: &egui_wgpu::RenderState) {
        let device = &render_state.device;

        let display_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Display Render Target"),
            size: wgpu::Extent3d {
                width: self.viewport_width,
                height: self.viewport_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let accum_size = (self.viewport_width as u64) * (self.viewport_height as u64) * 16;
        let accumulation_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("HDR Accumulation Buffer"),
            size: accum_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let display_view = display_texture.create_view(&wgpu::TextureViewDescriptor::default());

        if let Some(old_id) = self.render_target {
            render_state.renderer.write().free_texture(&old_id);
        }

        let texture_id = render_state.renderer.write().register_native_texture(
            device,
            &display_view,
            wgpu::FilterMode::Linear,
        );

        self.render_target = Some(texture_id);
        self.render_target_texture = Some(display_texture);
        self.accumulation_buffer = Some(accumulation_buffer);

        self.rebuild_bind_group(device);
        self.create_bloom_textures(render_state);

        self.frame_index = 0;
    }

    fn create_bloom_textures(&mut self, render_state: &egui_wgpu::RenderState) {
        let device = &render_state.device;
        let half_w = (self.viewport_width + 1) / 2;
        let half_h = (self.viewport_height + 1) / 2;

        let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING;

        let bloom_a = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Bloom A"),
            size: wgpu::Extent3d {
                width: half_w,
                height: half_h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage,
            view_formats: &[],
        });
        let bloom_a_view = bloom_a.create_view(&wgpu::TextureViewDescriptor::default());

        let bloom_b = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Bloom B"),
            size: wgpu::Extent3d {
                width: half_w,
                height: half_h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage,
            view_formats: &[],
        });
        let bloom_b_view = bloom_b.create_view(&wgpu::TextureViewDescriptor::default());

        self.bloom_a = Some(bloom_a);
        self.bloom_a_view = Some(bloom_a_view);
        self.bloom_b = Some(bloom_b);
        self.bloom_b_view = Some(bloom_b_view);

        self.rebuild_postfx_bind_groups(device);
    }

    fn rebuild_postfx_bind_groups(&mut self, device: &wgpu::Device) {
        let Some(accum_buf) = &self.accumulation_buffer else { return };
        let Some(display_tex) = &self.render_target_texture else { return };
        let display_view = display_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let Some(bloom_a_view) = &self.bloom_a_view else { return };
        let Some(bloom_b_view) = &self.bloom_b_view else { return };

        self.bloom_prefilter_bg = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Prefilter BG"),
            layout: &self.bloom_prefilter_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: accum_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(bloom_a_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.postfx_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.postfx_sampler),
                },
            ],
        }));

        self.bloom_blur_h_bg = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Blur H BG"),
            layout: &self.bloom_blur_h_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(bloom_a_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(bloom_b_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.postfx_sampler),
                },
            ],
        }));

        self.bloom_blur_v_bg = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Blur V BG"),
            layout: &self.bloom_blur_v_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(bloom_b_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(bloom_a_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.postfx_sampler),
                },
            ],
        }));

        self.postfx_composite_bg = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PostFX Composite BG"),
            layout: &self.postfx_composite_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: accum_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(bloom_a_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&display_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.postfx_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.postfx_sampler),
                },
            ],
        }));
    }

    fn update_camera_buffer(&self, queue: &wgpu::Queue) {
        let dir = self.camera.direction();
        let up = self.camera.up();

        let uniform = CameraUniform {
            pos: self.camera.position.into(),
            _pad1: 0.0,
            dir: dir.into(),
            _pad2: 0.0,
            up: up.into(),
            _pad3: 0.0,
            resolution: [self.viewport_width as f32, self.viewport_height as f32],
            sphere_count: self.scene.entities.len() as u32,
            frame_index: self.frame_index,
            fov_degrees: self.camera.fov,
            max_bounces: self.quality_state.max_bounces,
            samples_per_pixel: self.quality_state.samples_per_pixel,
            aperture: self.camera.aperture,
            focus_distance: self.camera.focus_distance,
            dof_enabled: self.camera.dof_enabled as u32,
            sun_enabled: self.sun_state.enabled as u32,
            sun_intensity: self.sun_state.intensity,
            sun_dir: self.sun_state.direction(),
            sun_angular_radius: self.sun_state.angular_radius_deg.to_radians(),
            sun_color: self.sun_state.color,
            floor_enabled: self.floor_state.enabled as u32,
            floor_color: self.floor_state.color,
            floor_roughness: self.floor_state.roughness,
            studio_top: self.skybox_state.studio_top,
            _pad4: 0.0,
            studio_bottom: self.skybox_state.studio_bottom,
            _pad5: 0.0,
            fog_color: self.fog_state.color,
            fog_enabled: self.fog_state.enabled as u32,
            _pad6: [0.0; 3],
            fog_density: self.fog_state.density,
            fill_dir: self.fill_state.direction(),
            fill_intensity: self.fill_state.intensity,
            fill_color: self.fill_state.color,
            fill_enabled: self.fill_state.enabled as u32,
            skybox_rotation: self.skybox_state.rotation_degrees.to_radians(),
            firefly_clamp: self.quality_state.firefly_clamp,
            floor_grid: self.floor_state.grid as u32,
            camera_roll: self.camera.roll,
            ambient_color: self.ambient_state.color,
            ambient_intensity: self.ambient_state.intensity,
            sky_intensity: self.skybox_state.sky_intensity,
            floor_height: self.floor_state.height,
            sun_shadows: self.sun_state.shadows as u32,
            _pad7: 0,
            floor_metallic: self.floor_state.metallic,
            floor_ior: self.floor_state.ior,
            floor_grid_scale: self.floor_state.grid_scale,
            floor_grid_thickness: self.floor_state.grid_thickness,
            floor_checker: self.floor_state.checker as u32,
            floor_uv_scale: self.floor_state.uv_scale,
            floor_emissive_intensity: self.floor_state.emissive_intensity,
            _pad8: 0.0,
            floor_grid_color: self.floor_state.grid_color,
            _pad9: 0.0,
            floor_emissive: self.floor_state.emissive,
            _pad10: 0.0,
        };

        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );
    }

    fn update_primitive_buffer(&self, queue: &wgpu::Queue) {
        let gpu_primitives = self.scene.to_gpu_primitives();
        if !gpu_primitives.is_empty() {
            queue.write_buffer(
                &self.primitive_buffer,
                0,
                bytemuck::cast_slice(&gpu_primitives),
            );
        }
    }

    fn update_skybox_buffer(&self, queue: &wgpu::Queue) {
        let uniform = SkyboxUniform {
            color: self.skybox_state.color,
            mode: self.skybox_state.mode,
        };
        queue.write_buffer(
            &self.skybox_buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );
    }

    pub fn update(&mut self, ctx: &Context, render_state: &egui_wgpu::RenderState) {
        if self.raytracing_enabled {
            ctx.request_repaint();
        }
        self.camera.update(ctx);
        self.postfx_time += ctx.input(|i| i.predicted_dt);

        let toggle_rt = ctx.input(|i| i.key_pressed(egui::Key::R) && !i.modifiers.ctrl && !i.modifiers.alt);
        if toggle_rt {
            self.raytracing_enabled = !self.raytracing_enabled;
            if self.raytracing_enabled {
                self.frame_index = 0;
            }
        }

        let camera_moved = self.camera.position != self.prev_camera_pos
            || self.camera.yaw != self.prev_camera_yaw
            || self.camera.pitch != self.prev_camera_pitch;

        if camera_moved {
            self.frame_index = 0;
            self.prev_camera_pos = self.camera.position;
            self.prev_camera_yaw = self.camera.yaw;
            self.prev_camera_pitch = self.camera.pitch;
        } else if self.raytracing_enabled {
            self.frame_index = self.frame_index.saturating_add(1);
        }

        theme::apply(ctx);

        self.draw_main_menu_bar(ctx, render_state);
        hierarchy::draw(ctx, &mut self.scene, &mut self.selected_entity_index);
        self.draw_properties_panel(ctx, render_state);
        self.draw_viewport(ctx, render_state);
        self.draw_status_bar(ctx);

        self.render(render_state);
    }

    fn render(&self, render_state: &egui_wgpu::RenderState) {
        let device = &render_state.device;
        let queue = &render_state.queue;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

        if self.raytracing_enabled {
            if let Some(bind_group) = &self.bind_group {
                self.update_camera_buffer(queue);
                self.update_primitive_buffer(queue);
                self.update_skybox_buffer(queue);

                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Raytracer Compute Pass"),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&self.compute_pipeline);
                compute_pass.set_bind_group(0, bind_group, &[]);
                let workgroups_x = (self.viewport_width + 15) / 16;
                let workgroups_y = (self.viewport_height + 15) / 16;
                compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
            }
        }

        let uniform = self.postfx_state.to_uniform(self.postfx_time);
        queue.write_buffer(&self.postfx_buffer, 0, bytemuck::cast_slice(&[uniform]));

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("PostFX Pass"),
                timestamp_writes: None,
            });

            if let Some(bg) = &self.bloom_prefilter_bg {
                pass.set_pipeline(&self.bloom_prefilter_pipeline);
                pass.set_bind_group(0, bg, &[]);
                let hw = (self.viewport_width / 2 + 7) / 8;
                let hh = (self.viewport_height / 2 + 7) / 8;
                pass.dispatch_workgroups(hw, hh, 1);
            }

            if let Some(bg) = &self.bloom_blur_h_bg {
                pass.set_pipeline(&self.bloom_blur_h_pipeline);
                pass.set_bind_group(0, bg, &[]);
                let hw = (self.viewport_width / 2 + 7) / 8;
                let hh = (self.viewport_height / 2 + 7) / 8;
                pass.dispatch_workgroups(hw, hh, 1);
            }

            if let Some(bg) = &self.bloom_blur_v_bg {
                pass.set_pipeline(&self.bloom_blur_v_pipeline);
                pass.set_bind_group(0, bg, &[]);
                let hw = (self.viewport_width / 2 + 7) / 8;
                let hh = (self.viewport_height / 2 + 7) / 8;
                pass.dispatch_workgroups(hw, hh, 1);
            }

            if let Some(bg) = &self.postfx_composite_bg {
                pass.set_pipeline(&self.postfx_composite_pipeline);
                pass.set_bind_group(0, bg, &[]);
                let hw = (self.viewport_width + 15) / 16;
                let hh = (self.viewport_height + 15) / 16;
                pass.dispatch_workgroups(hw, hh, 1);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn draw_postfx_content(&mut self, ui: &mut egui::Ui) {
        ui.add_space(2.0);

                ui.collapsing("Bloom", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_bloom, "Enable Bloom");
                    ui.add_enabled_ui(self.postfx_state.enabled_bloom, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.bloom_threshold, 0.0..=5.0).text("Threshold"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.bloom_soft_threshold, 0.0..=1.0).text("Soft Threshold"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.bloom_intensity, 0.0..=5.0).text("Intensity"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.bloom_luminance_bias, 0.0..=0.1).text("Luminance Bias"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.bloom_downsample_delta, 0.5..=2.0).text("Downsample Delta"));
                        ui.checkbox(&mut self.postfx_state.bloom_use_karis, "Use Karis Average");
                        ui.label("Blend Mode:");
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.postfx_state.bloom_blend_mode, 0u32, "Additive");
                            ui.radio_value(&mut self.postfx_state.bloom_blend_mode, 1u32, "Screen");
                        });
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.postfx_state.bloom_blend_mode, 2u32, "Darken");
                        });
                        ui.label("Bloom Tint:");
                        ui.color_edit_button_rgb(&mut self.postfx_state.bloom_tint);
                    });
                });

                ui.collapsing("Color Correction", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_color_correct, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_color_correct, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.exposure, 0.0..=10.0).text("Exposure"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.temperature, -1.0..=1.0).text("Temperature"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.tint, -1.0..=1.0).text("Tint"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.contrast, 0.0..=5.0).text("Contrast"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.linear_midpoint, 0.0..=1.0).text("Midpoint"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.brightness, -1.0..=1.0).text("Brightness"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.filter_intensity, 0.0..=5.0).text("Filter Intensity"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.saturation, 0.0..=5.0).text("Saturation"));
                        ui.label("Color Filter:");
                        ui.color_edit_button_rgb(&mut self.postfx_state.color_filter);
                    });
                });

                ui.collapsing("Vignette", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_vignette, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_vignette, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.vignette_intensity, 0.0..=5.0).text("Intensity"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.vignette_roundness, 0.0..=10.0).text("Roundness"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.vignette_smoothness, 0.0..=10.0).text("Smoothness"));
                        ui.label("Vignette Color:");
                        ui.color_edit_button_rgb(&mut self.postfx_state.vignette_color);
                        ui.add(egui::Slider::new(&mut self.postfx_state.vignette_size[0], 0.0..=3.0).text("Size X"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.vignette_size[1], 0.0..=3.0).text("Size Y"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.vignette_offset[0], -1.0..=1.0).text("Offset X"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.vignette_offset[1], -1.0..=1.0).text("Offset Y"));
                    });
                });

                ui.collapsing("Chromatic Aberration", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_chromatic_aberration, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_chromatic_aberration, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.ca_intensity, 0.0..=2.0).text("Intensity"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.ca_hardness, 0.0..=10.0).text("Hardness"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.ca_color_offsets[0], -2.0..=2.0).text("Offset R"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.ca_color_offsets[1], -2.0..=2.0).text("Offset G"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.ca_color_offsets[2], -2.0..=2.0).text("Offset B"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.ca_focal_offset[0], -1.0..=1.0).text("Focal Offset X"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.ca_focal_offset[1], -1.0..=1.0).text("Focal Offset Y"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.ca_radius[0], 0.0..=5.0).text("Radius X"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.ca_radius[1], 0.0..=5.0).text("Radius Y"));
                    });
                });

                ui.collapsing("Film Grain", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_grain, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_grain, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.grain_intensity, 0.0..=1.0).text("Intensity"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.grain_response, 0.0..=1.0).text("Response"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.grain_size, 1.0..=8.0).text("Grain Size"));
                        ui.checkbox(&mut self.postfx_state.animated_grain, "Animate");
                    });
                });

                ui.collapsing("Tone Mapping", |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.postfx_state.tonemap_mode, 0, "ACES");
                        ui.radio_value(&mut self.postfx_state.tonemap_mode, 1, "Reinhard");
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.postfx_state.tonemap_mode, 2, "Uncharted 2");
                        ui.radio_value(&mut self.postfx_state.tonemap_mode, 3, "Clamp");
                    });
                });

                ui.collapsing("Dithering", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_dither, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_dither, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.dither_spread, 0.0..=0.5).text("Spread"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.dither_color_counts[0], 2.0..=256.0).text("Color Count R"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.dither_color_counts[1], 2.0..=256.0).text("Color Count G"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.dither_color_counts[2], 2.0..=256.0).text("Color Count B"));
                    });
                });

                ui.collapsing("Sharpen", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_sharpen, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_sharpen, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.sharpen_amount, 0.0..=3.0).text("Amount"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.sharpen_radius, 0.5..=5.0).text("Radius"));
                    });
                });

                ui.collapsing("Pixelate", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_pixelate, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_pixelate, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.pixelate_size, 1.0..=64.0).text("Pixel Size"));
                    });
                });

                ui.collapsing("Scanlines", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_scanlines, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_scanlines, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.scanline_intensity, 0.0..=1.0).text("Intensity"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.scanline_frequency, 0.25..=8.0).text("Frequency"));
                    });
                });

                ui.collapsing("Edge Detection", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_edges, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_edges, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.edge_intensity, 0.0..=2.0).text("Intensity"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.edge_threshold, 0.0..=1.0).text("Threshold"));
                        ui.label("Edge Color:");
                        ui.color_edit_button_rgb(&mut self.postfx_state.edge_color);
                    });
                });

                ui.collapsing("Lens Distortion", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_distortion, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_distortion, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.distortion_k1, -1.0..=1.0).text("Barrel K1"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.distortion_k2, -1.0..=1.0).text("Barrel K2"));
                    });
                });

                ui.collapsing("Letterbox", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_letterbox, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_letterbox, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.letterbox_amount, 0.0..=0.45).text("Bar Size"));
                    });
                });

                ui.collapsing("Halation", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_halation, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_halation, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.halation_intensity, 0.0..=3.0).text("Intensity"));
                        ui.label("Halation Color:");
                        ui.color_edit_button_rgb(&mut self.postfx_state.halation_color);
                    });
                });

                ui.collapsing("Radial Blur", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_radial_blur, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_radial_blur, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.radial_intensity, 0.0..=1.0).text("Intensity"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.radial_center[0], 0.0..=1.0).text("Center X"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.radial_center[1], 0.0..=1.0).text("Center Y"));
                    });
                });

                ui.collapsing("Hue Shift", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_hue_shift, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_hue_shift, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.hue_shift_degrees, -180.0..=180.0).text("Hue"));
                    });
                });

                ui.collapsing("Lift / Gamma / Gain", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_lift_gamma_gain, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_lift_gamma_gain, |ui| {
                        ui.label("Lift (Shadows):");
                        ui.color_edit_button_rgb(&mut self.postfx_state.lift);
                        ui.label("Gain (Highlights):");
                        ui.color_edit_button_rgb(&mut self.postfx_state.gain);
                        ui.add(egui::Slider::new(&mut self.postfx_state.gamma_value, 0.2..=4.0).text("Gamma"));
                    });
                });

                ui.collapsing("Posterize", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_posterize, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_posterize, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.posterize_levels, 2.0..=32.0).text("Levels"));
                    });
                });

                ui.collapsing("Glitch (VHS)", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_glitch, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_glitch, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.glitch_intensity, 0.0..=1.0).text("Intensity"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.glitch_speed, 0.0..=30.0).text("Speed"));
                    });
                });

                ui.collapsing("Zoom", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_zoom, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_zoom, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.zoom_factor, 1.0..=5.0).text("Factor"));
                    });
                });

                ui.collapsing("Sepia", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_sepia, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_sepia, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.sepia_amount, 0.0..=1.0).text("Amount"));
                    });
                });

                ui.collapsing("Denoise", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_denoise, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_denoise, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.denoise_strength, 0.0..=1.0).text("Strength"));
                    });
                });

                ui.collapsing("Gaussian Blur", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_gaussian_blur, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_gaussian_blur, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.gaussian_radius, 0.5..=10.0).text("Radius"));
                    });
                });

                ui.collapsing("Kuwahara (Painterly)", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_kuwahara, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_kuwahara, |ui| {
                        let mut radius = self.postfx_state.kuwahara_radius as i32;
                        if ui.add(egui::Slider::new(&mut radius, 1..=4).text("Radius")).changed() {
                            self.postfx_state.kuwahara_radius = radius as f32;
                        }
                    });
                });

                ui.collapsing("Night Vision", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_nightvision, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_nightvision, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.nightvision_gain, 1.0..=30.0).text("Gain"));
                    });
                });

                ui.collapsing("Transform", |ui| {
                    ui.checkbox(&mut self.postfx_state.flip_horizontal, "Flip Horizontal");
                    ui.checkbox(&mut self.postfx_state.flip_vertical, "Flip Vertical");
                });

                ui.collapsing("Directional Blur", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_dir_blur, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_dir_blur, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.dirblur_angle, -180.0..=180.0).text("Angle"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.dirblur_distance, 0.0..=1.0).text("Distance"));
                    });
                });

                ui.collapsing("Tilt Shift", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_tilt_shift, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_tilt_shift, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.tiltshift_focus, 0.0..=1.0).text("Focus Y"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.tiltshift_range, 0.01..=0.5).text("Sharp Range"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.tiltshift_blur, 0.0..=1.0).text("Blur"));
                    });
                });

                ui.collapsing("Lens Flare", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_flare, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_flare, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.flare_intensity, 0.0..=3.0).text("Intensity"));
                        let mut ghosts = self.postfx_state.flare_ghosts;
                        if ui.add(egui::Slider::new(&mut ghosts, 1..=8).text("Ghosts")).changed() {
                            self.postfx_state.flare_ghosts = ghosts;
                        }
                    });
                });

                ui.collapsing("Water Warp", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_water_warp, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_water_warp, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.warp_amplitude, 0.0..=1.0).text("Amplitude"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.warp_frequency, 1.0..=20.0).text("Frequency"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.warp_speed, 0.0..=10.0).text("Speed"));
                    });
                });

                ui.collapsing("Thermal Vision", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_thermal, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_thermal, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.thermal_mix, 0.0..=1.0).text("Mix"));
                    });
                });

                ui.collapsing("Duotone", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_duotone, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_duotone, |ui| {
                        ui.label("Shadow Color:");
                        ui.color_edit_button_rgb(&mut self.postfx_state.duotone_shadow);
                        ui.label("Highlight Color:");
                        ui.color_edit_button_rgb(&mut self.postfx_state.duotone_highlight);
                        ui.add(egui::Slider::new(&mut self.postfx_state.duotone_amount, 0.0..=1.0).text("Amount"));
                    });
                });

                ui.collapsing("Halftone", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_halftone, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_halftone, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.halftone_size, 2.0..=32.0).text("Cell Size"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.halftone_mix, 0.0..=1.0).text("Mix"));
                    });
                });

                ui.collapsing("Old Film", |ui| {
                    ui.checkbox(&mut self.postfx_state.enabled_old_film, "Enable");
                    ui.add_enabled_ui(self.postfx_state.enabled_old_film, |ui| {
                        ui.add(egui::Slider::new(&mut self.postfx_state.film_scratches, 0.0..=1.0).text("Scratches"));
                        ui.add(egui::Slider::new(&mut self.postfx_state.film_flicker, 0.0..=1.0).text("Flicker"));
                    });
                });
    }

    fn draw_main_menu_bar(&mut self, ctx: &Context, render_state: &egui_wgpu::RenderState) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        self.scene = Scene::new();
                        self.selected_entity_index = None;
                        self.frame_index = 0;
                        ui.close_menu();
                    }
                    if ui.button("Save Render (PNG)").clicked() {
                        self.save_render_png(render_state);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save Scene (JSON)").clicked() {
                        self.save_scene_json();
                        ui.close_menu();
                    }
                    if ui.button("Load Scene (JSON)").clicked() {
                        self.load_scene_json();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Entity", |ui| {
                    if ui.button("Create Empty Entity").clicked() {
                        self.scene.add_empty_entity();
                        self.frame_index = 0;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Add Sphere").clicked() {
                        let count = self.scene.entities.iter().filter(|e| matches!(e.shape, ShapeType::Sphere)).count();
                        self.scene.add_sphere(&format!("Sphere {}", count));
                        self.frame_index = 0;
                        ui.close_menu();
                    }
                    if ui.button("Add Cube").clicked() {
                        let count = self.scene.entities.iter().filter(|e| matches!(e.shape, ShapeType::Cube)).count();
                        self.scene.add_cube(&format!("Cube {}", count));
                        self.frame_index = 0;
                        ui.close_menu();
                    }
                    if ui.button("Add Cylinder").clicked() {
                        let count = self.scene.entities.iter().filter(|e| matches!(e.shape, ShapeType::Cylinder)).count();
                        self.scene.add_cylinder(&format!("Cylinder {}", count));
                        self.frame_index = 0;
                        ui.close_menu();
                    }
                    if ui.button("Add Plane").clicked() {
                        let count = self.scene.entities.iter().filter(|e| matches!(e.shape, ShapeType::Plane)).count();
                        self.scene.add_plane(&format!("Plane {}", count));
                        self.frame_index = 0;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Randomize Scene").clicked() {
                        self.randomize_scene();
                        ui.close_menu();
                    }
                    ui.separator();
                    let has_selection = self.selected_entity_index.is_some();
                    ui.add_enabled_ui(has_selection, |ui| {
                        if ui.button("Duplicate Selected").clicked() {
                            if let Some(i) = self.selected_entity_index {
                                if i < self.scene.entities.len() {
                                    let src = &self.scene.entities[i];
                                    let mut transform = src.transform.clone();
                                    transform.position.x += 0.5;
                                    self.scene.add_entity(
                                        &format!("{} Copy", src.name),
                                        src.shape,
                                        transform,
                                        src.material.clone(),
                                    );
                                    self.frame_index = 0;
                                }
                            }
                            ui.close_menu();
                        }
                        if ui.button("Delete Selected").clicked() {
                            if let Some(i) = self.selected_entity_index {
                                self.scene.remove_entity(i);
                                self.selected_entity_index = None;
                                self.frame_index = 0;
                            }
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Render", |ui| {
                    let label = if self.raytracing_enabled {
                        "Disable Raytracing (R)"
                    } else {
                        "Enable Raytracing (R)"
                    };
                    if ui.button(label).clicked() {
                        self.raytracing_enabled = !self.raytracing_enabled;
                        if self.raytracing_enabled {
                            self.frame_index = 0;
                        }
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn draw_properties_panel(&mut self, ctx: &Context, render_state: &egui_wgpu::RenderState) {
        let mut insp_action = inspector::InspectorAction::None;
        let mut png_load: Option<std::path::PathBuf> = None;

        egui::SidePanel::right("properties_panel")
            .resizable(true)
            .default_width(300.0)
            .min_width(250.0)
            .show(ctx, |ui| {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.active_tab, 0, "Inspector");
                    ui.selectable_value(&mut self.active_tab, 1, "World");
                    ui.selectable_value(&mut self.active_tab, 2, "Post FX");
                });
                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match self.active_tab {
                            0 => {
                                insp_action = inspector::draw(ui, &mut self.scene, self.selected_entity_index);
                            }
                            1 => {
                                png_load = self.draw_world_content(ui);
                            }
                            _ => {
                                self.draw_postfx_content(ui);
                            }
                        }
                    });
            });

        if let Some(path) = png_load {
            self.load_skybox_image(render_state, &path);
        }

        match insp_action {
            inspector::InspectorAction::LoadTexture(entity_index) => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
                    .pick_file()
                {
                    self.load_entity_texture(render_state, entity_index, &path);
                }
            }
            inspector::InspectorAction::RemoveTexture(entity_index) => {
                self.remove_entity_texture(render_state, entity_index);
            }
            inspector::InspectorAction::ResetAccumulation => {
                self.frame_index = 0;
            }
            inspector::InspectorAction::None => {}
        }
    }

    fn draw_world_content(&mut self, ui: &mut egui::Ui) -> Option<std::path::PathBuf> {
        let mut needs_png_load: Option<std::path::PathBuf> = None;

        ui.add_space(2.0);

                ui.collapsing("Skybox", |ui| {
                    ui.label("Mode:");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.skybox_state.mode == 0, "Studio").clicked() {
                            self.skybox_state.mode = 0;
                            self.frame_index = 0;
                        }
                        if ui.selectable_label(self.skybox_state.mode == 1, "Color").clicked() {
                            self.skybox_state.mode = 1;
                            self.frame_index = 0;
                        }
                        if ui.selectable_label(self.skybox_state.mode == 2, "PNG").clicked() {
                            self.skybox_state.mode = 2;
                            self.frame_index = 0;
                        }
                    });

                    if self.skybox_state.mode == 0 {
                        ui.label("Gradient Top:");
                        let mut top: [f32; 3] = self.skybox_state.studio_top;
                        if ui.color_edit_button_rgb(&mut top).changed() {
                            self.skybox_state.studio_top = top;
                            self.frame_index = 0;
                        }
                        ui.label("Gradient Bottom:");
                        let mut bottom: [f32; 3] = self.skybox_state.studio_bottom;
                        if ui.color_edit_button_rgb(&mut bottom).changed() {
                            self.skybox_state.studio_bottom = bottom;
                            self.frame_index = 0;
                        }
                    }

                    if self.skybox_state.mode == 1 {
                        ui.label("Sky Color:");
                        let mut color: [f32; 3] = self.skybox_state.color;
                        if ui.color_edit_button_rgb(&mut color).changed() {
                            self.skybox_state.color = color;
                            self.frame_index = 0;
                        }
                    }

                    if self.skybox_state.mode == 2 {
                        ui.label("Environment Map:");
                        if let Some(id) = self.env_egui_id {
                            let available = ui.available_width();
                            let preview_size = available.min(180.0);
                            ui.image(egui::load::SizedTexture::new(id, egui::vec2(preview_size, preview_size)));
                        }
                        if ui.button("Load PNG...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "hdr"])
                                .pick_file()
                            {
                                needs_png_load = Some(path);
                            }
                        }
                    }

                    if self.skybox_state.mode != 1 {
                        if ui.add(egui::Slider::new(&mut self.skybox_state.rotation_degrees, 0.0..=360.0).text("Rotation")).changed() {
                            self.frame_index = 0;
                        }
                    }

                    if ui.add(egui::Slider::new(&mut self.skybox_state.sky_intensity, 0.0..=10.0).text("Sky Intensity")).changed() {
                        self.frame_index = 0;
                    }
                });

                ui.collapsing("Sun Light", |ui| {
                    if ui.checkbox(&mut self.sun_state.enabled, "Enable Sun").changed() {
                        self.frame_index = 0;
                    }
                    ui.add_enabled_ui(self.sun_state.enabled, |ui| {
                        if ui.add(egui::Slider::new(&mut self.sun_state.azimuth, 0.0..=360.0).text("Azimuth")).changed() {
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.sun_state.elevation, -10.0..=90.0).text("Elevation")).changed() {
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.sun_state.intensity, 0.0..=20.0).text("Intensity")).changed() {
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.sun_state.angular_radius_deg, 0.1..=30.0).text("Softness")).changed() {
                            self.frame_index = 0;
                        }
                        let mut color: [f32; 3] = self.sun_state.color;
                        if ui.color_edit_button_rgb(&mut color).changed() {
                            self.sun_state.color = color;
                            self.frame_index = 0;
                        }
                        if ui.checkbox(&mut self.sun_state.shadows, "Cast Shadows").changed() {
                            self.frame_index = 0;
                        }
                    });
                });

                ui.collapsing("Fill Light", |ui| {
                    if ui.checkbox(&mut self.fill_state.enabled, "Enable Fill Light").changed() {
                        self.frame_index = 0;
                    }
                    ui.add_enabled_ui(self.fill_state.enabled, |ui| {
                        if ui.add(egui::Slider::new(&mut self.fill_state.azimuth, 0.0..=360.0).text("Azimuth")).changed() {
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.fill_state.elevation, -10.0..=90.0).text("Elevation")).changed() {
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.fill_state.intensity, 0.0..=10.0).text("Intensity")).changed() {
                            self.frame_index = 0;
                        }
                        let mut color: [f32; 3] = self.fill_state.color;
                        if ui.color_edit_button_rgb(&mut color).changed() {
                            self.fill_state.color = color;
                            self.frame_index = 0;
                        }
                    });
                });

                ui.collapsing("Ambient Light", |ui| {
                    if ui.add(egui::Slider::new(&mut self.ambient_state.intensity, 0.0..=2.0).text("Intensity")).changed() {
                        self.frame_index = 0;
                    }
                    let mut color: [f32; 3] = self.ambient_state.color;
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        self.ambient_state.color = color;
                        self.frame_index = 0;
                    }
                });

                ui.collapsing("Floor", |ui| {
                    if ui.checkbox(&mut self.floor_state.enabled, "Enable Floor").changed() {
                        self.frame_index = 0;
                    }
                    ui.add_enabled_ui(self.floor_state.enabled, |ui| {
                        let mut color: [f32; 3] = self.floor_state.color;
                        if ui.color_edit_button_rgb(&mut color).changed() {
                            self.floor_state.color = color;
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.floor_state.roughness, 0.0..=1.0).text("Roughness")).changed() {
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.floor_state.metallic, 0.0..=1.0).text("Metallic")).changed() {
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.floor_state.ior, 1.0..=2.5).text("IOR / Reflectivity")).changed() {
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.floor_state.uv_scale, 0.1..=10.0).text("Pattern Scale")).changed() {
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.floor_state.height, -5.0..=2.0).text("Height")).changed() {
                            self.frame_index = 0;
                        }

                        ui.collapsing("Grid", |ui| {
                            if ui.checkbox(&mut self.floor_state.grid, "Show Grid").changed() {
                                self.frame_index = 0;
                            }
                            if ui.checkbox(&mut self.floor_state.checker, "Checkerboard").changed() {
                                self.frame_index = 0;
                            }
                            if ui.add(egui::Slider::new(&mut self.floor_state.grid_scale, 0.1..=10.0).text("Cell Size")).changed() {
                                self.frame_index = 0;
                            }
                            if ui.add(egui::Slider::new(&mut self.floor_state.grid_thickness, 0.01..=0.4).text("Line Thickness")).changed() {
                                self.frame_index = 0;
                            }
                            let mut gcolor: [f32; 3] = self.floor_state.grid_color;
                            if ui.color_edit_button_rgb(&mut gcolor).changed() {
                                self.floor_state.grid_color = gcolor;
                                self.frame_index = 0;
                            }
                        });

                        ui.collapsing("Emissive", |ui| {
                            if ui.add(egui::Slider::new(&mut self.floor_state.emissive_intensity, 0.0..=10.0).text("Intensity")).changed() {
                                self.frame_index = 0;
                            }
                            let mut ecolor: [f32; 3] = self.floor_state.emissive;
                            if ui.color_edit_button_rgb(&mut ecolor).changed() {
                                self.floor_state.emissive = ecolor;
                                self.frame_index = 0;
                            }
                        });
                    });
                });

                ui.collapsing("Atmosphere", |ui| {
                    if ui.checkbox(&mut self.fog_state.enabled, "Enable Fog").changed() {
                        self.frame_index = 0;
                    }
                    ui.add_enabled_ui(self.fog_state.enabled, |ui| {
                        if ui.add(egui::Slider::new(&mut self.fog_state.density, 0.0..=0.5).text("Density")).changed() {
                            self.frame_index = 0;
                        }
                        let mut color: [f32; 3] = self.fog_state.color;
                        if ui.color_edit_button_rgb(&mut color).changed() {
                            self.fog_state.color = color;
                            self.frame_index = 0;
                        }
                    });
                });

                ui.collapsing("Camera", |ui| {
                    if ui.add(egui::Slider::new(&mut self.camera.fov, 10.0..=120.0).text("FOV")).changed() {
                        self.frame_index = 0;
                    }
                    if ui.add(egui::Slider::new(&mut self.camera.roll, -180.0..=180.0).text("Roll")).changed() {
                        self.frame_index = 0;
                    }
                    if ui.checkbox(&mut self.camera.dof_enabled, "Depth of Field").changed() {
                        self.frame_index = 0;
                    }
                    ui.add_enabled_ui(self.camera.dof_enabled, |ui| {
                        if ui.add(egui::Slider::new(&mut self.camera.aperture, 0.0..=1.0).text("Aperture")).changed() {
                            self.frame_index = 0;
                        }
                        if ui.add(egui::Slider::new(&mut self.camera.focus_distance, 0.5..=50.0).text("Focus Distance")).changed() {
                            self.frame_index = 0;
                        }
                    });
                });

                ui.collapsing("Quality", |ui| {
                    let mut bounces = self.quality_state.max_bounces as i32;
                    if ui.add(egui::Slider::new(&mut bounces, 1..=12).text("Max Bounces")).changed() {
                        self.quality_state.max_bounces = bounces as u32;
                        self.frame_index = 0;
                    }
                    let mut spp = self.quality_state.samples_per_pixel as i32;
                    if ui.add(egui::Slider::new(&mut spp, 1..=16).text("Samples / Pixel")).changed() {
                        self.quality_state.samples_per_pixel = spp as u32;
                        self.frame_index = 0;
                    }
                    if ui.add(egui::Slider::new(&mut self.render_scale, 0.25..=1.0).text("Render Scale")).changed() {
                        self.frame_index = 0;
                    }
                    if ui.add(egui::Slider::new(&mut self.quality_state.firefly_clamp, 0.5..=100.0).text("Firefly Clamp")).changed() {
                        self.frame_index = 0;
                    }
                });

                ui.separator();
                ui.label(format!("Sky: {}", match self.skybox_state.mode {
                    0 => "Studio",
                    1 => "Color",
                    2 => "PNG",
                    _ => "Unknown",
                }));

        needs_png_load
    }

    fn load_skybox_image(&mut self, render_state: &egui_wgpu::RenderState, path: &std::path::Path) {
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                self.upload_env_texture(render_state, &rgba, w, h);
            }
            Err(e) => {
                eprintln!("Failed to load skybox image: {}", e);
            }
        }
    }

    fn randomize_scene(&mut self) {
        let mut rng = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() ^ ((d.subsec_nanos() as u64) << 32))
            .unwrap_or(42);

        self.scene.entities.clear();

        for i in 0..14 {
            let angle = lcg_next(&mut rng) * std::f32::consts::TAU;
            let dist = 0.8 + lcg_next(&mut rng) * 3.5;
            let position = Vec3::new(
                angle.cos() * dist,
                -0.6 + lcg_next(&mut rng) * 1.8,
                angle.sin() * dist,
            );
            let radius = 0.25 + lcg_next(&mut rng) * 0.55;

            let kind = lcg_next(&mut rng);
            let tint = Vec3::new(
                0.15 + lcg_next(&mut rng) * 0.85,
                0.15 + lcg_next(&mut rng) * 0.85,
                0.15 + lcg_next(&mut rng) * 0.85,
            );

            let material = if kind < 0.55 {
                MaterialComponent {
                    albedo: tint,
                    roughness: 0.1 + lcg_next(&mut rng) * 0.9,
                    ..MaterialComponent::default()
                }
            } else if kind < 0.72 {
                MaterialComponent {
                    albedo: tint,
                    metallic: 1.0,
                    roughness: 0.03 + lcg_next(&mut rng) * 0.35,
                    ..MaterialComponent::default()
                }
            } else if kind < 0.88 {
                MaterialComponent {
                    albedo: Vec3::new(0.98, 0.98, 1.0),
                    transmission: 0.92,
                    roughness: 0.02,
                    ior: 1.45 + lcg_next(&mut rng) * 0.15,
                    clearcoat: 0.5,
                    ..MaterialComponent::default()
                }
            } else {
                MaterialComponent {
                    albedo: Vec3::ZERO,
                    emissive: tint * (2.0 + lcg_next(&mut rng) * 6.0),
                    ..MaterialComponent::default()
                }
            };

            self.scene.add_entity(
                &format!("Sphere {}", i),
                ShapeType::Sphere,
                TransformComponent {
                    position,
                    rotation: Vec3::ZERO,
                    scale: Vec3::splat(radius),
                },
                material,
            );
        }

        self.selected_entity_index = None;
        self.frame_index = 0;
    }

    fn scene_to_dto(&self) -> SceneDto {
        SceneDto {
            entities: self
                .scene
                .entities
                .iter()
                .map(|e| EntityDto {
                    name: e.name.clone(),
                    shape: shape_to_string(&e.shape).to_string(),
                    position: e.transform.position.into(),
                    rotation: e.transform.rotation.into(),
                    scale: e.transform.scale.into(),
                    material: MaterialDto {
                        albedo: e.material.albedo.into(),
                        emissive: e.material.emissive.into(),
                        roughness: e.material.roughness,
                        metallic: e.material.metallic,
                        ior: e.material.ior,
                        opacity: e.material.opacity,
                        visible: e.material.visible,
                        cast_shadow: e.material.cast_shadow,
                        two_sided: e.material.two_sided,
                        texture_id: e.material.texture_id,
                        clearcoat: e.material.clearcoat,
                        sheen: e.material.sheen,
                        transmission: e.material.transmission,
                        emissive_intensity: e.material.emissive_intensity,
                        specular_tint: e.material.specular_tint,
                        uv_scale: e.material.uv_scale.into(),
                        uv_offset: e.material.uv_offset.into(),
                    },
                })
                .collect(),
        }
    }

    fn save_scene_json(&self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Scene", &["json"])
            .set_file_name("scene.json")
            .save_file()
        else {
            return;
        };

        match serde_json::to_string_pretty(&self.scene_to_dto()) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    eprintln!("Failed to save scene: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to serialize scene: {}", e),
        }
    }

    fn load_scene_json(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Scene", &["json"])
            .pick_file()
        else {
            return;
        };

        let json = match std::fs::read_to_string(&path) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Failed to read scene file: {}", e);
                return;
            }
        };

        let dto: SceneDto = match serde_json::from_str(&json) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to parse scene file: {}", e);
                return;
            }
        };

        self.scene.entities.clear();
        for ed in dto.entities {
            let m = ed.material;
            self.scene.add_entity(
                &ed.name,
                shape_from_string(&ed.shape),
                TransformComponent {
                    position: ed.position.into(),
                    rotation: ed.rotation.into(),
                    scale: ed.scale.into(),
                },
                MaterialComponent {
                    albedo: m.albedo.into(),
                    emissive: m.emissive.into(),
                    roughness: m.roughness,
                    metallic: m.metallic,
                    ior: m.ior,
                    opacity: m.opacity,
                    visible: m.visible,
                    cast_shadow: m.cast_shadow,
                    two_sided: m.two_sided,
                    texture_id: 0,
                    clearcoat: m.clearcoat,
                    sheen: m.sheen,
                    transmission: m.transmission,
                    emissive_intensity: m.emissive_intensity,
                    specular_tint: m.specular_tint,
                    uv_scale: m.uv_scale.into(),
                    uv_offset: m.uv_offset.into(),
                },
            );
        }

        self.selected_entity_index = None;
        self.frame_index = 0;
    }

    fn save_render_png(&self, render_state: &egui_wgpu::RenderState) {
        let device = &render_state.device;
        let queue = &render_state.queue;
        let Some(texture) = &self.render_target_texture else { return };

        let w = self.viewport_width;
        let h = self.viewport_height;
        let bytes_per_row = ((w * 4 + 255) / 256) * 256;

        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PNG Readback Buffer"),
            size: (bytes_per_row * h) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("PNG Readback Encoder"),
        });

        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &readback,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(std::iter::once(encoder.finish()));

        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        let _ = device.poll(wgpu::Maintain::Wait);
        if let Err(e) = rx.recv() {
            eprintln!("Readback map failed: {}", e);
            return;
        }

        let data = slice.get_mapped_range();
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for row in 0..h {
            let start = (row * bytes_per_row) as usize;
            let end = start + (w * 4) as usize;
            rgba.extend_from_slice(&data[start..end]);
        }
        drop(data);
        readback.unmap();

        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("render.png")
            .add_filter("PNG Image", &["png"])
            .save_file()
        {
            match image::save_buffer(&path, &rgba, w, h, image::ColorType::Rgba8) {
                Ok(_) => println!("Saved render to {}", path.display()),
                Err(e) => eprintln!("Failed to save PNG: {}", e),
            }
        }
    }

    pub fn load_entity_texture(&mut self, render_state: &egui_wgpu::RenderState, entity_index: usize, path: &std::path::Path) {
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let resized = image::imageops::resize(&rgba, ENTITY_TEX_SIZE, ENTITY_TEX_SIZE, image::imageops::FilterType::Lanczos3);

                let slot = match self.entity_texture_used.iter().position(|&used| !used) {
                    Some(s) => s,
                    None => {
                        eprintln!("No free texture slots available (max {})", MAX_ENTITY_TEXTURES);
                        return;
                    }
                };

                let queue = &render_state.queue;
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: self.entity_texture_array.as_ref().unwrap(),
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: 0, y: 0, z: slot as u32 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &resized,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * ENTITY_TEX_SIZE),
                        rows_per_image: Some(ENTITY_TEX_SIZE),
                    },
                    wgpu::Extent3d {
                        width: ENTITY_TEX_SIZE,
                        height: ENTITY_TEX_SIZE,
                        depth_or_array_layers: 1,
                    },
                );

                if let Some(entity) = self.scene.entities.get_mut(entity_index) {
                    entity.material.texture_id = slot as u32 + 1;
                }
                self.entity_texture_used[slot] = true;
                self.frame_index = 0;
            }
            Err(e) => {
                eprintln!("Failed to load entity texture: {}", e);
            }
        }
    }

    pub fn remove_entity_texture(&mut self, render_state: &egui_wgpu::RenderState, entity_index: usize) {
        if let Some(entity) = self.scene.entities.get_mut(entity_index) {
            let tex_id = entity.material.texture_id;
            if tex_id > 0 {
                let slot = (tex_id - 1) as usize;
                if slot < self.entity_texture_used.len() {
                    self.entity_texture_used[slot] = false;

                    let white_pixel: [u8; 4] = [255, 255, 255, 255];
                    let data: Vec<u8> = white_pixel.iter().copied().cycle().take((ENTITY_TEX_SIZE * ENTITY_TEX_SIZE * 4) as usize).collect();
                    render_state.queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: self.entity_texture_array.as_ref().unwrap(),
                            mip_level: 0,
                            origin: wgpu::Origin3d { x: 0, y: 0, z: slot as u32 },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &data,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * ENTITY_TEX_SIZE),
                            rows_per_image: Some(ENTITY_TEX_SIZE),
                        },
                        wgpu::Extent3d {
                            width: ENTITY_TEX_SIZE,
                            height: ENTITY_TEX_SIZE,
                            depth_or_array_layers: 1,
                        },
                    );

                    entity.material.texture_id = 0;
                    self.frame_index = 0;
                }
            }
        }
    }

    fn draw_viewport(&mut self, ctx: &Context, render_state: &egui_wgpu::RenderState) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_black_alpha(0))
                    .inner_margin(0.0),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    let rt_label = if self.raytracing_enabled { "Pause (R)" } else { "Resume (R)" };
                    if ui.button(rt_label).clicked() {
                        self.raytracing_enabled = !self.raytracing_enabled;
                        if self.raytracing_enabled {
                            self.frame_index = 0;
                        }
                    }
                    if ui.button("Reset View").clicked() {
                        self.frame_index = 0;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(6.0);
                        ui.weak(format!("{}x{}", self.viewport_width, self.viewport_height));
                        ui.weak(format!("{:.0}%", self.render_scale * 100.0));
                        ui.weak(format!("frame {}", self.frame_index));
                    });
                });
                ui.separator();

                let available_size = ui.available_size();
                let scale = self.render_scale.clamp(0.1, 1.0);
                let new_width = ((available_size.x * scale) as u32).max(1);
                let new_height = ((available_size.y * scale) as u32).max(1);

                if new_width > 0
                    && new_height > 0
                    && (new_width != self.viewport_width || new_height != self.viewport_height)
                {
                    self.viewport_width = new_width;
                    self.viewport_height = new_height;
                    self.create_viewport_textures(render_state);
                }

                if let Some(texture_id) = self.render_target {
                    ui.image(egui::load::SizedTexture::new(
                        texture_id,
                        available_size,
                    ));
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Initializing 3D Viewport...");
                    });
                }
            });
    }

    fn draw_status_bar(&self, ctx: &Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(6.0);
                    ui.weak("WASD move · RMB+drag look · Space/C up/down · R raytracing");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(6.0);
                        let dt = ctx.input(|i| i.predicted_dt);
                        let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };
                        ui.weak(format!("{:.0} fps", fps));
                        ui.weak(format!(
                            "{} spp · {} bounces",
                            self.quality_state.samples_per_pixel, self.quality_state.max_bounces
                        ));
                        ui.weak(format!("{} entities", self.scene.entities.len()));
                        ui.weak(format!(
                            "cam ({:.1}, {:.1}, {:.1})",
                            self.camera.position.x, self.camera.position.y, self.camera.position.z
                        ));
                        if !self.raytracing_enabled {
                            ui.weak("paused");
                        }
                    });
                });
            });
    }
}
