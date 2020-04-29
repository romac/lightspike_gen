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

    let scheduler = scheduler::handler(scheduler::process);
    let verifier = verifier::handler(verifier::process);
    let io = io::handler(io::process);

    let mut demuxer = Demuxer::new(state, scheduler, verifier, io);

    let result = demuxer.verify_height(42);
    dbg!(&result);
}
