use crate::scene::{ShapeType, Scene};
use egui::Context;

pub enum InspectorAction {
    None,
    LoadTexture(usize),
    RemoveTexture(usize),
    ResetAccumulation,
}

pub fn draw(ctx: &Context, scene: &mut Scene, selected_index: Option<usize>) -> InspectorAction {
    let mut action = InspectorAction::None;

    egui::SidePanel::right("inspector")
        .resizable(true)
        .default_width(260.0)
        .show(ctx, |ui| {
            ui.heading("Inspector");
            ui.separator();

            if let Some(index) = selected_index {
                if let Some(entity) = scene.entities.get_mut(index) {
                    let transform_before = entity.transform.clone();
                    let material_before = entity.material.clone();

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut entity.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Shape:");
                        ui.label(entity.shape.name());
                    });

                    ui.separator();

                    egui::CollapsingHeader::new("Transform")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Position");
                                ui.add(egui::DragValue::new(&mut entity.transform.position.x).speed(0.1));
                                ui.add(egui::DragValue::new(&mut entity.transform.position.y).speed(0.1));
                                ui.add(egui::DragValue::new(&mut entity.transform.position.z).speed(0.1));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Rotation");
                                ui.add(egui::DragValue::new(&mut entity.transform.rotation.x).speed(0.1));
                                ui.add(egui::DragValue::new(&mut entity.transform.rotation.y).speed(0.1));
                                ui.add(egui::DragValue::new(&mut entity.transform.rotation.z).speed(0.1));
                            });

                            let scale_label = match entity.shape {
                                ShapeType::Sphere => "Radius",
                                ShapeType::Cube => "Size",
                                ShapeType::Cylinder => "Rad/Ht",
                                ShapeType::Plane => "Extents",
                            };

                            ui.horizontal(|ui| {
                                ui.label(scale_label);
                                ui.add(egui::DragValue::new(&mut entity.transform.scale.x).speed(0.1));
                                if matches!(entity.shape, ShapeType::Cube | ShapeType::Cylinder | ShapeType::Plane) {
                                    ui.add(egui::DragValue::new(&mut entity.transform.scale.y).speed(0.1));
                                }
                                if matches!(entity.shape, ShapeType::Cube | ShapeType::Plane) {
                                    ui.add(egui::DragValue::new(&mut entity.transform.scale.z).speed(0.1));
                                }
                            });
                        });

                    ui.separator();

                    egui::CollapsingHeader::new("Material")
                        .default_open(true)
                        .show(ui, |ui| {
                            let mut albedo = entity.material.albedo.into();
                            ui.horizontal(|ui| {
                                ui.label("Albedo");
                                ui.color_edit_button_rgb(&mut albedo);
                            });
                            entity.material.albedo = albedo.into();

                            let mut emissive = entity.material.emissive.into();
                            ui.horizontal(|ui| {
                                ui.label("Emissive");
                                ui.color_edit_button_rgb(&mut emissive);
                            });
                            entity.material.emissive = emissive.into();

                            ui.horizontal(|ui| {
                                ui.label("Emissive Intensity");
                                ui.add(
                                    egui::Slider::new(&mut entity.material.emissive_intensity, 0.0..=20.0)
                                        .clamp_to_range(true),
                                );
                            });

                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("Roughness");
                                ui.add(
                                    egui::Slider::new(&mut entity.material.roughness, 0.0..=1.0)
                                        .clamp_to_range(true),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Metallic");
                                ui.add(
                                    egui::Slider::new(&mut entity.material.metallic, 0.0..=1.0)
                                        .clamp_to_range(true),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Specular Tint");
                                ui.add(
                                    egui::Slider::new(&mut entity.material.specular_tint, 0.0..=2.0)
                                        .clamp_to_range(true),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Clearcoat");
                                ui.add(
                                    egui::Slider::new(&mut entity.material.clearcoat, 0.0..=1.0)
                                        .clamp_to_range(true),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Sheen");
                                ui.add(
                                    egui::Slider::new(&mut entity.material.sheen, 0.0..=2.0)
                                        .clamp_to_range(true),
                                );
                            });

                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("Transmission");
                                ui.add(
                                    egui::Slider::new(&mut entity.material.transmission, 0.0..=1.0)
                                        .clamp_to_range(true),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("IOR");
                                ui.add(
                                    egui::Slider::new(&mut entity.material.ior, 1.0..=2.5)
                                        .clamp_to_range(true),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Opacity");
                                ui.add(
                                    egui::Slider::new(&mut entity.material.opacity, 0.0..=1.0)
                                        .clamp_to_range(true),
                                );
                            });

                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("UV Scale");
                                ui.add(egui::DragValue::new(&mut entity.material.uv_scale.x).speed(0.05));
                                ui.add(egui::DragValue::new(&mut entity.material.uv_scale.y).speed(0.05));
                            });

                            ui.horizontal(|ui| {
                                ui.label("UV Offset");
                                ui.add(egui::DragValue::new(&mut entity.material.uv_offset.x).speed(0.01));
                                ui.add(egui::DragValue::new(&mut entity.material.uv_offset.y).speed(0.01));
                            });

                            ui.separator();
                            ui.label("Texture:");

                            if entity.material.texture_id > 0 {
                                ui.label(format!("  Slot {} loaded", entity.material.texture_id - 1));
                                if ui.button("Remove Texture").clicked() {
                                    action = InspectorAction::RemoveTexture(index);
                                }
                            }

                            if ui.button("Import PNG...").clicked() {
                                action = InspectorAction::LoadTexture(index);
                            }

                            ui.separator();

                            ui.checkbox(&mut entity.material.visible, "Visible");
                            ui.checkbox(&mut entity.material.cast_shadow, "Cast Shadow");
                            ui.checkbox(&mut entity.material.two_sided, "Two-Sided");
                        });

                    if matches!(action, InspectorAction::None) {
                        if transform_before != entity.transform || material_before != entity.material {
                            action = InspectorAction::ResetAccumulation;
                        }
                    }
                }
            } else {
                ui.label("No entity selected.");
            }
        });

    action
}
