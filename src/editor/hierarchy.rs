use crate::editor::theme;
use crate::scene::Scene;
use egui::Context;

pub fn draw(ctx: &Context, scene: &mut Scene, selected_index: &mut Option<usize>) {
    egui::SidePanel::left("scene_hierarchy")
        .resizable(true)
        .default_width(200.0)
        .show(ctx, |ui| {
            theme::panel_title(ui, "Hierarchy");

            ui.add_space(2.0);

            let entity_count = scene.entities.len();
            let mut action_delete: Option<usize> = None;
            let mut action_duplicate: Option<usize> = None;
            let mut action_up: Option<usize> = None;
            let mut action_down: Option<usize> = None;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for i in 0..entity_count {
                        let name = scene.entities[i].name.clone();
                        let is_selected = *selected_index == Some(i);
                        let response = ui.selectable_label(is_selected, &name);

                        if response.clicked() {
                            *selected_index = Some(i);
                        }

                        response.context_menu(|ui| {
                            if ui.button("Delete").clicked() {
                                action_delete = Some(i);
                                ui.close_menu();
                            }
                            if ui.button("Duplicate").clicked() {
                                action_duplicate = Some(i);
                                ui.close_menu();
                            }
                            ui.separator();
                            if i > 0 && ui.button("Move Up").clicked() {
                                action_up = Some(i);
                                ui.close_menu();
                            }
                            if i < entity_count - 1 && ui.button("Move Down").clicked() {
                                action_down = Some(i);
                                ui.close_menu();
                            }
                        });
                    }

                    if entity_count == 0 {
                        ui.weak("Scene is empty");
                    }
                });

            if let Some(idx) = action_delete {
                scene.entities.remove(idx);
                *selected_index = if scene.entities.is_empty() {
                    None
                } else if idx >= scene.entities.len() {
                    Some(scene.entities.len() - 1)
                } else {
                    Some(idx)
                };
            } else if let Some(idx) = action_duplicate {
                let (s, t, m, n) = {
                    let e = &scene.entities[idx];
                    (e.shape, e.transform.clone(), e.material.clone(), e.name.clone())
                };
                scene.add_entity(&format!("{} (copy)", n), s, t, m);
                *selected_index = Some(scene.entities.len() - 1);
            } else if let Some(idx) = action_up {
                scene.entities.swap(idx, idx - 1);
                *selected_index = Some(idx - 1);
            } else if let Some(idx) = action_down {
                scene.entities.swap(idx, idx + 1);
                *selected_index = Some(idx + 1);
            }
        });
}
