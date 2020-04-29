#![allow(unused_variables, unused_imports)]

use lightspike_gen::demuxer::Demuxer;
use lightspike_gen::prelude::*;

pub fn main() {
    let store = TrustedStore::new();
    let (trusted_store_reader, trusted_store_writer) = store.split();

    let state = State {
        trusted_store_reader,
        trusted_store_writer,
    };

    let mut demuxer = Demuxer::new(state);

    let result = demuxer.verify_height(42);
    dbg!(&result);
}
