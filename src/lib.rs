#![allow(incomplete_features)]
#![feature(iter_array_chunks)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(generic_const_exprs)]
#![feature(iter_from_generator)]
#![feature(arbitrary_self_types)]
#![feature(c_size_t)]

mod atlas;
mod camera;
#[cfg(feature = "egui")]
mod egui;
mod instance;
mod layer;
mod packing;
#[cfg(feature = "py")]
mod py;
mod renderer;
mod sprite;
#[cfg(feature = "standalone")]
mod standalone;
mod vertex;
#[cfg(feature = "standalone")]
mod c;

pub use atlas::*;
pub use camera::*;
pub use instance::*;
pub use layer::*;
pub use renderer::*;
pub use sprite::*;

#[cfg(feature = "standalone")]
pub use standalone::*;
