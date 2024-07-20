#[cfg(feature = "sync")]
pub mod blocking;

mod config;

#[cfg(feature = "async")]
pub mod nonblocking;
