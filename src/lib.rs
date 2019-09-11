extern crate rand;
#[macro_use]
extern crate lazy_static;

/* use rand::Rng; */

pub mod util;
pub mod cbms;
pub mod wbms;
pub mod compiler;
pub mod cbms_printer;
#[cfg(test)]
mod tests;