#![allow(incomplete_features)]
#![feature(iter_array_chunks)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(generic_const_exprs)]
#![feature(iter_from_generator)]

mod atlas;
mod camera;
mod packing;
mod instance;
mod renderer;
mod sprite;
mod vertex;
mod egui;
mod standalone;

pub use atlas::*;
pub use camera::*;
pub use instance::*;
pub use renderer::*;
pub use sprite::*;
pub use standalone::*;
