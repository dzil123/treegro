#![forbid(unsafe_code)]

use treegro::{cell::Cell, *};

use rand::prelude::*;

const NUM_CELLS: u32 = WIDTH * HEIGHT;

#[derive(Default)]
struct World {
    cells: Vec<Cell>,
    time_delta: f32,
    growth_rate: f32,
}

impl World {
    fn new() -> Self {
        let mut this = Self {
            ..Default::default()
        };
        this.randomize();
        this
    }

    fn randomize(&mut self) {
        self.cells = (0..NUM_CELLS)
            .map(|_| Cell {
                resources: 0.5,
                density: random(),
            })
            .collect();
    }

    fn update_cells(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.step(self.growth_rate, self.time_delta);
        }
    }
}

impl App for World {
    fn update(&mut self, frame: &mut [u8], ctx: &egui::Context) {
        egui::Window::new("TreeGro").show(ctx, |ui| {
            if ui.button("Randomize").clicked() {
                self.randomize();
            }

            ui.add(egui::Slider::new(&mut self.growth_rate, 0.0..=4.0));

            ui.add(egui::Slider::new(&mut self.time_delta, 0.1..=2.0));
        });

        self.update_cells();

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
