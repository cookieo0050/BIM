mod editor;
mod scene;

use editor::SceneEditor;
use eframe::egui;
use eframe::egui_wgpu;

pub struct App {
    editor: SceneEditor,
    render_state: egui_wgpu::RenderState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.editor.update(ctx, &self.render_state);
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "FIGLORD EDITOR!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!",
        native_options,
        Box::new(|cc| {
            let render_state = cc
                .wgpu_render_state
                .as_ref()
                .expect("WGPU render state required")
                .clone();

            let editor = SceneEditor::new(&render_state);
            Box::new(App {
                editor,
                render_state,
            })
        }),
    )
}

#[cfg(test)]
mod tests {
    fn parse(source: &str) {
        let mut frontend = naga::front::wgsl::Frontend::new();
        let result = frontend.parse(source);
        assert!(result.is_ok(), "WGSL parse error: {:?}", result.err());
    }

    #[test]
    fn validate_raytracer_shader() {
        parse(include_str!("shader.wgsl"));
    }

    #[test]
    fn validate_postprocess_shader() {
        parse(include_str!("postprocess.wgsl"));
    }
}
