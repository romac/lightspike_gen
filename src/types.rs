pub type Height = u64;

#[derive(Copy, Clone, Debug)]
pub struct LightBlock;

#[derive(Copy, Clone, Debug)]
pub struct State;

impl State {
    pub fn is_trusted(&self, height: Height) -> bool {
        rand::random()
    }

    pub fn get_trusted_state(&self, height: Height) -> Option<LightBlock> {
        if self.is_trusted(height) {
            Some(LightBlock)
        } else {
            None
        }
    }

    pub fn add_trusted_states(&self, trusted_states: Vec<LightBlock>) {
        ()
    }

    pub fn add_valid_light_block(&self, light_block: LightBlock) {
        ()
    }

    pub fn add_fetched_light_block(&self, light_block: LightBlock) {
        ()
    }
}
