#![forbid(unsafe_code)]

use egui::emath::Numeric as Num;
use std::ops::RangeInclusive;
use ultraviolet::Vec4;

pub mod cell;
mod fsm;
mod gui;
mod mainloop;
pub mod param;
mod world;

pub use mainloop::{mainloop, App, HEIGHT, WIDTH};
pub use world::{World, NUM_RESOURCES};

pub fn random_vec4() -> ultraviolet::Vec4 {
    use rand::random;

    ultraviolet::Vec4::new(random(), random(), random(), random())
}

pub fn lerp_vec4(start: Vec4, end: Vec4, t: Vec4) -> Vec4 {
    (Vec4::one() - t) * start + t * end
}

pub fn ui_vec(ui: &mut egui::Ui, row: &mut [f32], range: RangeInclusive<impl Num>) -> bool {
    let mut changed = false;
    for val in row {
        let mut response = ui.add(
            egui::widgets::DragValue::new(val)
                .clamp_range(range.clone())
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
        changed |= response.changed();
    }
    changed
}
