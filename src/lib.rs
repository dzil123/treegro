#![forbid(unsafe_code)]

pub mod cell;
mod gui;
mod mainloop;

pub use mainloop::{mainloop, App, HEIGHT, WIDTH};

pub fn random_vec4() -> ultraviolet::Vec4 {
    use rand::random;

    ultraviolet::Vec4::new(random(), random(), random(), random())
}
