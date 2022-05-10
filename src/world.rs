use bytemuck::Zeroable;
use pixels::Pixels;
use ultraviolet::{Mat4, Vec4};

use crate::param::ResourceVector;
use crate::{cell::Cell, *};

pub const NUM_RESOURCES: usize = 4;

#[derive(Default)]
pub struct World {
    cells: Vec<Cell>,
    cells_tmp: Vec<Cell>,
    running: bool,
    absolute_value: bool,
    time_delta: f32,
    ticks_per_frame: i16, // if negative, then frames per tick
    tick_timer: u8,
    matrix: Mat4,
    diffuse: f32,
    diffuse_enabled: bool,
    diffuse_strength: Vec4,
    pixels_size: (u32, u32),
    snapshot: Vec<Vec<Cell>>,
    snapshot_enabled: bool,
    pub resources: ResourceVector,
}

impl World {
    pub fn new() -> Self {
        let mut this = Self {
            pixels_size: (WIDTH, HEIGHT),
            ticks_per_frame: 1,
            time_delta: 0.01,
            matrix: Mat4::zeroed(),
            diffuse: 0.2,
            diffuse_strength: Vec4::one(),
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

    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        let i = (x + (self.pixels_size.0 as i32) * y) as isize;
        if 0 <= i && i < (self.cells.len() as isize) {
            Some(i as usize)
        } else {
            None
        }
    }

    fn diffuse_pass(&mut self) {
        let amount = (self.diffuse * self.diffuse_strength).clamped(Vec4::zero(), Vec4::one());
        self.cells_tmp.clone_from(&self.cells);

        // really inefficient
        for x in 0..(self.pixels_size.0 as i32) {
            for y in 0..(self.pixels_size.1 as i32) {
                let mut sum = Vec4::zero();
                let mut total = 0;
                for dx in -1..=1i32 {
                    for dy in -1..=1i32 {
                        if let Some(i) = self.idx(x + dx, y + dy) {
                            total += 1;
                            sum += self.cells[i].density;
                        }
                    }
                }
                let i = self.idx(x, y).unwrap();
                let current = self.cells[i].density;
                let average = sum / (total as f32);
                let new_density = lerp_vec4(current, average, amount);
                self.cells_tmp[i].density = new_density;
            }
        }

        std::mem::swap(&mut self.cells, &mut self.cells_tmp);
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
            if self.diffuse_enabled {
                self.diffuse_pass();
            }
            if self.snapshot_enabled {
                self.snapshot.push(self.cells.clone());
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
                        egui::Slider::from_get_set(-64.0..=4096.0, |val| {
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
                            changed_matrix |= ui_vec(ui, row, -2.0..=2.0);
                            ui.end_row();
                        }
                    });
                });

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Diffuse");
                        ui.checkbox(&mut self.diffuse_enabled, "Enabled");
                    });
                    ui.add(egui::Slider::new(&mut self.diffuse, 0.0..=1.0).logarithmic(true));
                    ui.horizontal(|ui| {
                        ui_vec(ui, self.diffuse_strength.as_mut_slice(), 0.0..=2.0);
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

        egui::Window::new("Plot").show(ctx, |ui| {
            ui.checkbox(&mut self.snapshot_enabled, "Enabled");
            if ui.button("Clear").clicked() {
                self.snapshot.clear();
            }

            use egui::plot::*;
            Plot::new("Snapshot").show(ui, |plot_ui| {
                let coord = 0; // todo
                let densities: Vec<Vec4> = self
                    .snapshot
                    .iter()
                    .map(|cells| cells[coord].density)
                    .collect();

                for i in 0..4 {
                    let values = Values::from_values_iter(
                        densities
                            .iter()
                            .map(|density| density[i])
                            .enumerate()
                            .map(|(x, y)| Value::new(x as f64, y)),
                    );

                    plot_ui.line(Line::new(values));
                }
            });
        });

        if changed_size {
            pixels.resize_buffer(self.pixels_size.0, self.pixels_size.1);
            self.randomize();
            self.snapshot.clear();
            self.snapshot_enabled = false;
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
