#![forbid(unsafe_code)]

use bytemuck::Zeroable;
use ultraviolet::{Mat4, Vec4};

use pixels::Pixels;
use treegro::{mainloop, World};

fn main() {
    let world = World::new();
    mainloop(world);
}
