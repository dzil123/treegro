#![forbid(unsafe_code)]

pub mod cell;
mod gui;
mod mainloop;

pub use mainloop::{mainloop, App, HEIGHT, WIDTH};
