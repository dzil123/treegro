#![forbid(unsafe_code)]

#[derive(Default)]
struct World {}

impl World {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl treegro::App for World {
    fn update(&mut self, frame: &mut [u8], ctx: &egui::Context) {
        egui::Window::new("Hello, egui!").show(ctx, |ui| {
            ui.label("This example demonstrates using egui with pixels.");
            ui.label("Made with 💖 in San Francisco!");

            ui.separator();

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x /= 2.0;
            });
        });
    }
}

fn main() {
    let world = World::new();
    treegro::mainloop(world)
}
