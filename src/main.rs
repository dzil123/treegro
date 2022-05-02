#![forbid(unsafe_code)]

use pixels::Pixels;
use treegro::{cell::Cell, *};

use rand::prelude::*;

#[derive(Default)]
struct World {
    cells: Vec<Cell>,
    time_delta: f32,
    growth_rate: f32,

    pixels_size: (u32, u32),
}

impl World {
    fn new() -> Self {
        let mut this = Self {
            pixels_size: (WIDTH, HEIGHT),
            ..Default::default()
        };
        this.randomize();
        this
    }

    fn randomize(&mut self) {
        let num_cells = self.pixels_size.0 * self.pixels_size.1;
        self.cells.clear();
        self.cells.resize_with(num_cells as _, || Cell {
            resources: 0.5,
            density: random(),
        });
    }

    fn update_cells(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.step(self.growth_rate, self.time_delta);
        }
    }
}

impl App for World {
    fn update(&mut self, pixels: &mut Pixels, ctx: &egui::Context) {
        let mut changed_size = false;

        egui::Window::new("TreeGro").show(ctx, |ui| {
            if ui.button("Randomize").clicked() {
                self.randomize();
            }

            ui.add(egui::Slider::new(&mut self.growth_rate, 0.0..=4.0));

            ui.add(egui::Slider::new(&mut self.time_delta, 0.1..=2.0));

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
            });
        });

        if changed_size {
            pixels.resize_buffer(self.pixels_size.0, self.pixels_size.1);
            self.randomize();
            return;
        }

        self.update_cells();

        let frame = pixels.get_frame();
        for (cell, pixel) in self.cells.iter().zip(frame.chunks_exact_mut(4)) {
            let f = (cell.density * 256.0) as u8;
            pixel.copy_from_slice(&[f, f, f, 0xFF]);
        }
    }
}

fn main() {
    let world = World::new();
    mainloop(world)
}
