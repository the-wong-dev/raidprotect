//! Protocol used to communicate between services.
//!
//! This crate contains types used to communicate between services.
//! Communication is based on a TCP connection with [`remoc`] channels.

pub mod model;
pub mod server;