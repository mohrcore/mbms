extern crate num;
use num::integer::Integer;
use std::str::FromStr;
use std::fmt::{Display, Formatter};
use std::error::Error;

pub fn pair_diff<T>(pair: (T, T)) -> T where T: Integer {
    pair.1 - pair.0
}

#[derive(Debug)]
pub struct GenericError {
    error_string: String,
}

impl Display for GenericError {
    fn fmt(&self, f:  &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.error_string)
    }
}

impl Error for GenericError {
    fn description(&self) -> &str {
        &self.error_string
    } 
}

impl FromStr for GenericError {
    type Err = std::io::Error;
    fn from_str(s: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            error_string: s.to_string(),
        })
    }
}