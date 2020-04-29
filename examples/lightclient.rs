#![allow(unused_variables, unused_imports)]

pub use gentest::prelude::*;

pub fn main() {
    let mut state = State;
    let result = demuxer::verify_height(&mut state, 42);
    dbg!(&result);
}

