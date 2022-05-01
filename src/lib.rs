#![forbid(unsafe_code)]

mod gui;
mod mainloop;

pub use mainloop::{mainloop, App, HEIGHT, WIDTH};
