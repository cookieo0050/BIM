use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeType {
    Sphere,
    Cube,
    Cylinder,
    Plane,
}

impl ShapeType {
    pub fn name(&self) -> &'static str {
        match self {
            ShapeType::Sphere => "Sphere",
            ShapeType::Cube => "Cube",
            ShapeType::Cylinder => "Cylinder",
            ShapeType::Plane => "Plane",
        }
    }

    pub fn from_index(i: u32) -> Self {
        match i {
            0 => ShapeType::Sphere,
            1 => ShapeType::Cube,
            2 => ShapeType::Cylinder,
            3 => ShapeType::Plane,
            _ => ShapeType::Sphere,
        }
    }

    pub fn index(&self) -> u32 {
        match self {
            ShapeType::Sphere => 0,
            ShapeType::Cube => 1,
            ShapeType::Cylinder => 2,
            ShapeType::Plane => 3,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuPrimitive {
    pub shape_type: u32,
    pub _pad0: [u32; 3],
    pub position: [f32; 3],
    pub _pad1: f32,
    pub size: [f32; 3],
    pub _pad2: f32,
    pub rotation: [f32; 3],
    pub _pad3: f32,
    pub albedo: [f32; 3],
    pub _pad4: f32,
    pub emissive: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
    pub ior: f32,
    pub opacity: f32,
    pub flags: u32,
    pub texture_id: u32,
    pub uv_scale_x: f32,
    pub uv_scale_y: f32,
    pub uv_offset_x: f32,
    pub uv_offset_y: f32,
    pub clearcoat: f32,
    pub sheen: f32,
    pub transmission: f32,
    pub emissive_intensity: f32,
    pub specular_tint: f32,
    pub _pad5: [f32; 2],
}

impl GpuPrimitive {
    pub fn from_entity(e: &Entity) -> Self {
        let t = &e.transform;
        let m = &e.material;
        let shape = e.shape.index();
        let mut flags: u32 = 0;
        if e.material.visible { flags |= 1; }
        if e.material.cast_shadow { flags |= 2; }
        if e.material.two_sided { flags |= 4; }

        let size = match e.shape {
            ShapeType::Sphere => [t.scale.x, 0.0, 0.0],
            ShapeType::Cube => [t.scale.x, t.scale.y, t.scale.z],
            ShapeType::Cylinder => [t.scale.x, t.scale.y, 0.0],
            ShapeType::Plane => [t.scale.x, t.scale.y, t.scale.z],
        };

        GpuPrimitive {
            shape_type: shape,
            _pad0: [0; 3],
            position: t.position.into(),
            _pad1: 0.0,
            size,
            _pad2: 0.0,
            rotation: t.rotation.into(),
            _pad3: 0.0,
            albedo: m.albedo.into(),
            _pad4: 0.0,
            emissive: m.emissive.into(),
            roughness: m.roughness,
            metallic: m.metallic,
            ior: m.ior,
            opacity: m.opacity,
            flags,
            texture_id: m.texture_id,
            uv_scale_x: m.uv_scale.x,
            uv_scale_y: m.uv_scale.y,
            uv_offset_x: m.uv_offset.x,
            uv_offset_y: m.uv_offset.y,
            clearcoat: m.clearcoat,
            sheen: m.sheen,
            transmission: m.transmission,
            emissive_intensity: m.emissive_intensity,
            specular_tint: m.specular_tint,
            _pad5: [0.0; 2],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialComponent {
    pub albedo: Vec3,
    pub emissive: Vec3,
    pub roughness: f32,
    pub metallic: f32,
    pub ior: f32,
    pub opacity: f32,
    pub visible: bool,
    pub cast_shadow: bool,
    pub two_sided: bool,
    pub texture_id: u32,
    pub clearcoat: f32,
    pub sheen: f32,
    pub transmission: f32,
    pub emissive_intensity: f32,
    pub specular_tint: f32,
    pub uv_scale: Vec2,
    pub uv_offset: Vec2,
}

impl Default for MaterialComponent {
    fn default() -> Self {
        Self {
            albedo: Vec3::new(0.8, 0.8, 0.8),
            emissive: Vec3::ZERO,
            roughness: 0.5,
            metallic: 0.0,
            ior: 1.5,
            opacity: 1.0,
            visible: true,
            cast_shadow: true,
            two_sided: false,
            texture_id: 0,
            clearcoat: 0.0,
            sheen: 0.0,
            transmission: 0.0,
            emissive_intensity: 1.0,
            specular_tint: 1.0,
            uv_scale: Vec2::ONE,
            uv_offset: Vec2::ZERO,
        }
    }
}

pub struct Entity {
    #[allow(dead_code)]
    pub id: u32,
    pub name: String,
    pub shape: ShapeType,
    pub transform: TransformComponent,
    pub material: MaterialComponent,
}

pub struct Scene {
    pub entities: Vec<Entity>,
    next_id: u32,
}

impl Scene {
    pub fn new() -> Self {
        let mut scene = Self {
            entities: Vec::new(),
            next_id: 0,
        };

        let default_spheres = [
            ("Sphere 0", Vec3::new(0.0, -0.3, 0.0), 0.7, Vec3::new(0.5, 0.08, 0.65), Vec3::ZERO, 0.08, 0.2, 1.5),
            ("Sphere 1", Vec3::new(1.8, 0.5, 1.2), 1.3, Vec3::new(0.9, 0.15, 0.15), Vec3::ZERO, 0.12, 0.05, 1.5),
            ("Sphere 2", Vec3::new(-1.8, -0.2, -1.0), 0.8, Vec3::new(0.1, 0.45, 0.95), Vec3::ZERO, 0.06, 0.1, 1.5),
            ("Sphere 3", Vec3::new(-1.7, 0.4, 1.8), 1.0, Vec3::new(0.55, 0.05, 0.85), Vec3::ZERO, 0.08, 0.05, 1.5),
            ("Sphere 4", Vec3::new(0.9, -0.5, -0.8), 0.45, Vec3::new(0.4, 0.1, 0.85), Vec3::ZERO, 0.05, 0.05, 1.5),
            ("Sphere 5", Vec3::new(2.0, -0.4, -0.5), 0.5, Vec3::new(0.1, 0.85, 0.25), Vec3::ZERO, 0.05, 0.05, 1.5),
            ("Sphere 6", Vec3::new(0.3, 0.1, 2.0), 0.7, Vec3::new(0.08, 0.35, 0.95), Vec3::ZERO, 0.06, 0.05, 1.5),
            ("Sphere 7", Vec3::new(-0.9, -0.3, 0.6), 0.4, Vec3::new(0.95, 0.1, 0.1), Vec3::ZERO, 0.05, 0.05, 1.5),
            ("Sphere 8", Vec3::new(-0.6, -0.4, 0.2), 0.3, Vec3::new(0.1, 0.85, 0.25), Vec3::new(2.0, 1.5, 0.5), 0.05, 0.05, 1.5),
        ];

        for (name, pos, radius, albedo, emissive, roughness, metallic, ior) in default_spheres {
            scene.add_entity(
                name,
                ShapeType::Sphere,
                TransformComponent {
                    position: pos,
                    rotation: Vec3::ZERO,
                    scale: Vec3::splat(radius),
                },
                MaterialComponent {
                    albedo,
                    emissive,
                    roughness,
                    metallic,
                    ior,
                    ..MaterialComponent::default()
                },
            );
        }

        // Demo glass sphere
        scene.entities[0].material.transmission = 0.92;
        scene.entities[0].material.roughness = 0.02;
        scene.entities[0].material.clearcoat = 0.5;

        scene
    }

    pub fn add_entity(
        &mut self,
        name: &str,
        shape: ShapeType,
        transform: TransformComponent,
        material: MaterialComponent,
    ) -> &Entity {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.push(Entity {
            id,
            name: name.to_string(),
            shape,
            transform,
            material,
        });
        &self.entities.last().unwrap()
    }

    pub fn add_sphere(&mut self, name: &str) {
        self.add_entity(
            name,
            ShapeType::Sphere,
            TransformComponent {
                position: Vec3::ZERO,
                rotation: Vec3::ZERO,
                scale: Vec3::splat(0.5),
            },
            MaterialComponent::default(),
        );
    }

    pub fn add_cube(&mut self, name: &str) {
        self.add_entity(
            name,
            ShapeType::Cube,
            TransformComponent {
                position: Vec3::ZERO,
                rotation: Vec3::ZERO,
                scale: Vec3::splat(0.5),
            },
            MaterialComponent::default(),
        );
    }

    pub fn add_cylinder(&mut self, name: &str) {
        self.add_entity(
            name,
            ShapeType::Cylinder,
            TransformComponent {
                position: Vec3::ZERO,
                rotation: Vec3::ZERO,
                scale: Vec3::new(0.5, 0.5, 0.5),
            },
            MaterialComponent::default(),
        );
    }

    pub fn add_plane(&mut self, name: &str) {
        self.add_entity(
            name,
            ShapeType::Plane,
            TransformComponent {
                position: Vec3::ZERO,
                rotation: Vec3::ZERO,
                scale: Vec3::new(5.0, 5.0, 1.0),
            },
            MaterialComponent::default(),
        );
    }

    pub fn add_empty_entity(&mut self) {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.push(Entity {
            id,
            name: format!("Entity {}", id),
            shape: ShapeType::Sphere,
            transform: TransformComponent::default(),
            material: MaterialComponent::default(),
        });
    }

    #[allow(dead_code)]
    pub fn remove_entity(&mut self, index: usize) -> Option<Entity> {
        if index < self.entities.len() {
            Some(self.entities.remove(index))
        } else {
            None
        }
    }

    pub fn to_gpu_primitives(&self) -> Vec<GpuPrimitive> {
        self.entities.iter().map(GpuPrimitive::from_entity).collect()
    }
}
