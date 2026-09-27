//! The wire types, shaped exactly like the TypeSafe API.
//!
//! Every map is an `IndexMap` because order means something: questions go
//! out in the order you wrote them, options in the order the model reads
//! them, and level 0 is the bottom of the scale

pub mod answers;
pub mod entry;
pub mod jevvy_error;
pub mod jevvy_request;
pub mod jevvy_response;
pub mod questions;
pub mod typed;

pub use entry::*;
