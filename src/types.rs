use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub type Height = u64;

#[derive(Copy, Clone, Debug)]
pub struct LightBlock;

#[derive(Debug)]
pub struct State {
    pub trusted_store_reader: TSReader,
    pub trusted_store_writer: TSReadWriter,
    // valid_store_reader: TSReader,
    // valid_store_writer: TSReaderWriter,
    // fetched_store_reader: TSReader,
    // fetched_store_writer: TSReaderWriter,
}

impl State {
    pub fn trusted_store_reader(&self) -> TSReader {
        self.trusted_store_reader.clone()
    }

    pub fn add_trusted_states(&mut self, trusted_states: Vec<LightBlock>) {
        for trusted_state in trusted_states {
            self.trusted_store_writer.add(trusted_state);
        }
    }

    pub fn add_valid_light_block(&mut self, light_block: LightBlock) {
        // self.valid_store_writer.add(light_block);
    }

    pub fn add_fetched_light_block(&mut self, light_block: LightBlock) {
        // self.fetched_store_writer.add(light_block);
    }
}

#[derive(Debug, Default)]
pub struct TrustedStore {
    store: HashMap<Height, LightBlock>,
}

impl TrustedStore {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn split(self) -> (TSReader, TSReadWriter) {
        let store = Arc::new(RwLock::new(self));
        let reader = TSReader { ts: store.clone() };
        let writer = TSReadWriter { ts: store };

        (reader, writer)
    }
}

impl TrustedStore {
    pub fn get(&self, height: Height) -> Option<&LightBlock> {
        self.store.get(&height)
    }

    pub fn add(&mut self, trusted_state: LightBlock) {
        let height = 0; // Stub
        self.store.insert(height, trusted_state);
    }
}

#[derive(Clone, Debug)]
pub struct TSReader {
    ts: Arc<RwLock<TrustedStore>>,
}

impl TSReader {
    pub fn get(&self, height: Height) -> Option<LightBlock> {
        self.ts.read().unwrap().get(height).cloned()
    }
}

#[derive(Debug)]
pub struct TSReadWriter {
    ts: Arc<RwLock<TrustedStore>>,
}

impl TSReadWriter {
    pub fn get(&self, height: Height) -> Option<LightBlock> {
        self.ts.read().unwrap().get(height).cloned()
    }

    pub fn add(&mut self, trusted_state: LightBlock) {
        let mut ts = self.ts.write().unwrap();
        ts.add(trusted_state);
    }
}
