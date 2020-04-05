#![allow(dead_code)]

#[cfg(windows)]
extern crate gfx_backend_dx12 as backend;
#[cfg(target_os = "macos")]
extern crate gfx_backend_metal as backend;
#[cfg(all(unix, not(target_os = "macos")))]
extern crate gfx_backend_vulkan as backend;

pub extern crate image;
extern crate gfx_hal;
extern crate nalgebra_glm as glm;
extern crate winit;

pub mod color;
pub mod input;
pub mod mesh;
pub mod renderer;
pub mod sprite;
pub mod texture;
pub mod window;

pub use gfx_hal::image::*;

pub type Vector2f = nalgebra::Vector2::<f32>;
pub type Point2f = nalgebra::Point2::<f32>;
pub type Point2u = nalgebra::Point2::<u32>;
