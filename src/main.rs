#![forbid(unsafe_code)]

use treegro::*;

use rand::prelude::*;

const NUM_CELLS: u32 = WIDTH * HEIGHT;

#[derive(Clone, Default)]
struct Cell {
    resources: f32,
}

#[derive(Default)]
struct World {
    cells: Vec<Cell>,
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
                resources: random(),
            })
            .collect();
    }
}

impl App for World {
    fn update(&mut self, frame: &mut [u8], ctx: &egui::Context) {
        egui::Window::new("TreeGro").show(ctx, |ui| {
            if ui.button("Randomize").clicked() {
                self.randomize();
            }
        });

        for (cell, pixel) in self.cells.iter().zip(frame.chunks_exact_mut(4)) {
            let f = (cell.resources * 256.0) as u8;
            pixel.copy_from_slice(&[f, f, f, 0xFF]);
        }
    }
}

fn main() {
    let world = World::new();
    mainloop(world)
}
