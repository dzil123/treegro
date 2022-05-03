#![forbid(unsafe_code)]

use bytemuck::Zeroable;
use ultraviolet::{Mat4, Vec4};

use pixels::Pixels;
use treegro::{cell::Cell, *};

#[derive(Default)]
struct World {
    cells: Vec<Cell>,
    running: bool,
    absolute_value: bool,
    time_delta: f32,
    ticks_per_frame: i8, // if negative, then frames per tick
    tick_timer: u8,
    matrix: Mat4,
    pixels_size: (u32, u32),
}

impl World {
    fn new() -> Self {
        let mut this = Self {
            pixels_size: (WIDTH, HEIGHT),
            ticks_per_frame: 1,
            time_delta: 0.01,
            matrix: Mat4::zeroed(),
            ..Self::default()
        };
        this.randomize();
        this
    }

    fn randomize(&mut self) {
        let num_cells = self.pixels_size.0 * self.pixels_size.1;
        self.cells.clear();
        self.cells.resize_with(num_cells as _, || Cell {
            resources: Vec4::broadcast(0.5),
            density: random_vec4(),
        });
    }

    fn update_cells(&mut self) {
        let count = if self.ticks_per_frame < 0 {
            self.tick_timer += 1;
            if self.tick_timer >= -self.ticks_per_frame as _ {
                self.tick_timer = 0;
                1
            } else {
                0
            }
        } else {
            self.ticks_per_frame
        };
        for _ in 0..count {
            for cell in &mut self.cells {
                cell.step(self.matrix, self.time_delta);
            }
        }
    }
}

impl App for World {
    fn init(&mut self, _pixels: &mut Pixels, ctx: &egui::Context) {
        // make the windows slightly transparent
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.bg_fill =
            egui::Color32::from_rgba_premultiplied(27, 27, 27, 245);
        ctx.set_visuals(visuals);
    }

    fn update(&mut self, pixels: &mut Pixels, ctx: &egui::Context) {
        let mut changed_size = false;

        let mut isolate_color = [false; 4];

        egui::Window::new("TreeGro")
            // egui::SidePanel::left("TreeGro")
            .show(ctx, |ui| {
                ui.group(|ui| {
                    ui.checkbox(&mut self.running, "Running");
                    ui.checkbox(&mut self.absolute_value, "Absolute Value");
                    if ui.button("Randomize").clicked() {
                        self.randomize();
                    }

                    ui.horizontal(|ui| {
                        for (i, text) in ["Red", "Green", "Blue", "Alpha"].into_iter().enumerate() {
                            isolate_color[i] = ui
                                .add(egui::Button::new(text).sense(egui::Sense::drag()))
                                .dragged();
                        }
                    });
                });

                ui.group(|ui| {
                    ui.add(
                        egui::Slider::new(&mut self.time_delta, 0.01..=2.0)
                            .text("dt")
                            .logarithmic(true),
                    );

                    ui.add(
                        egui::Slider::from_get_set(-64.0..=64.0, |val| {
                            if let Some(v) = val {
                                let v = v.round() as _;
                                self.ticks_per_frame = if v == -1 || v == 0 { 1 } else { v }
                            }

                            self.ticks_per_frame as _
                        })
                        .text("Ticks")
                        .integer()
                        .logarithmic(true),
                    );
                });

                let mut changed_matrix = false;
                ui.group(|ui| {
                    ui.label("Weights Matrix")
                        .on_hover_text("Right click to set to 0\nMiddle click to set to 1");
                    egui::Grid::new("Matrix").show(ui, |ui| {
                        for row in self.matrix.as_mut_slice().chunks_exact_mut(4) {
                            for val in row {
                                let mut response = ui.add(
                                    egui::widgets::DragValue::new(val)
                                        .clamp_range(-2.0..=2.0)
                                        .fixed_decimals(2)
                                        .speed(0.01),
                                );
                                if response.clicked_by(egui::PointerButton::Secondary) {
                                    *val = 0.0;
                                    response.mark_changed();
                                }
                                if response.clicked_by(egui::PointerButton::Middle) {
                                    *val = 1.0;
                                    response.mark_changed();
                                }
                                changed_matrix |= response.changed();
                            }
                            ui.end_row();
                        }
                    });
                });

                if changed_matrix {
                    dbg!(self.matrix);
                }

                ui.group(|ui| {
                    changed_size |= ui
                        .add(
                            egui::Slider::new(&mut self.pixels_size.0, 1..=WIDTH * 2)
                                .text("Width")
                                .logarithmic(true),
                        )
                        .changed();
                    changed_size |= ui
                        .add(
                            egui::Slider::new(&mut self.pixels_size.1, 1..=HEIGHT * 2)
                                .text("Height")
                                .logarithmic(true),
                        )
                        .changed();

                    ui.horizontal(|ui| {
                        for (text, size) in [
                            ("Small", (8, 8)),
                            ("Medium", (64, 64)),
                            ("Large", (WIDTH, HEIGHT)),
                        ] {
                            if ui.button(text).clicked() {
                                self.pixels_size = size;
                                changed_size |= true;
                            }
                        }
                    });
                });
            });

        if changed_size {
            pixels.resize_buffer(self.pixels_size.0, self.pixels_size.1);
            self.randomize();
            return;
        }

        if self.running {
            self.update_cells();
        }

        let mut isolate_color_mat = Mat4::identity();
        for (i, c) in isolate_color.into_iter().enumerate() {
            if c {
                isolate_color_mat = Mat4::zeroed();
                isolate_color_mat.cols[i] = Vec4::one();
            }
        }

        let frame = pixels.get_frame();
        for (cell, pixel) in self.cells.iter().zip(frame.chunks_exact_mut(4)) {
            let mut f = cell.density * 256.0;
            f = isolate_color_mat * f;
            if self.absolute_value {
                f = f.abs();
            }
            pixel.copy_from_slice(&[f.x as _, f.y as _, f.z as _, 0xFF]);
        }
    }
}

fn main() {
    let world = World::new();
    mainloop(world);
}
