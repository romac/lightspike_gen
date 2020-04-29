#![allow(unused_variables, unused_imports)]

pub use gentest::prelude::*;

pub fn main() {
    let store = TrustedStore::new();
    let (trusted_store_reader, trusted_store_writer) = store.split();

    let mut state = State {
        trusted_store_reader,
        trusted_store_writer,
    };

    let result = demuxer::verify_height(&mut state, 42);
    dbg!(&result);
}

