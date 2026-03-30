#![forbid(unstable_features)]

#![recursion_limit = "256"]


#[macro_use]
extern crate derive_new;

pub mod data;
mod embedding;
pub mod fill_mask;
pub mod loader;
pub mod model;
pub mod pooler;
pub mod show_prediction;
