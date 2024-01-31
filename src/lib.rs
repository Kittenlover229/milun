#![allow(incomplete_features)]
#![feature(iter_array_chunks)]
#![feature(coroutines)]
#![feature(trait_alias)]
#![feature(allocator_api)]
#![feature(coroutine_trait)]
#![feature(generic_const_exprs)]
#![feature(iter_from_coroutine)]
#![feature(arbitrary_self_types)]
#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(c_size_t)]
#![feature(ptr_metadata)]

mod atlas;
mod camera;
mod frame;
#[cfg(feature = "egui")]
mod egui;
mod ffi;
mod instance;
mod layer;
mod packing;
mod renderer;
mod sprite;
#[cfg(feature = "standalone")]
mod standalone;
mod vertex;

pub use atlas::*;
pub use camera::*;
#[allow(unused_imports)]
pub use ffi::*;
pub use instance::*;
pub use layer::*;
pub use renderer::*;
pub use sprite::*;
pub use frame::*;

#[cfg(feature = "standalone")]
pub use standalone::*;
