#![allow(unused_imports)]

use genawaiter::{
    rc::Co,
    yield_, GeneratorState,
};

pub type Height = u64;

pub struct LightBlock;
pub struct TrustedState;

pub struct State;

impl State {
    pub fn is_trusted(&self, height: Height) -> bool {
        rand::random()
    }

    pub fn get_trusted_state(&self, height: Height) -> Option<TrustedState> {
        if self.is_trusted() {
        Some(TrustedState)
        } else { None}
    }
}

pub enum DemuxerInput {}
pub enum DemuxerOutput {}
pub enum DemuxerError {
    Scheduler(SchedulerError),
    Verifier(VerifierError),
    Io(IoError),
}

pub type DemuxerResult = Result<DemuxerOutput, DemuxerError>;

pub async fn demuxer() {
    let mut _state = State {};

    loop {
        // TODO
    }
}

pub enum SchedulerInput {
    VerifyHeight(Height),
    VerifyLightBlock(LightBlock),
}

pub enum SchedulerOutput {
    TrustedState(TrustedState),
}

pub enum SchedulerError {}

pub type SchedulerResult = Result<SchedulerOutput, SchedulerError>;

pub async fn scheduler(state: &State, input: SchedulerInput, co: Co<SchedulerResult>) {
    match input {
        SchedulerInput::VerifyHeight(height) => {
            if let Some(ts) = state.get_trusted_state(height) {
                co.yield_(Ok(SchedulerOutput::TrustedState(ts))).await;
            } else {
                co.yield_(Ok(SchedulerOutput::RequestLightBlock(height))).await;
            }
        }
    }
}

pub enum VerifierInput {
    VerifyLightBlock(LightBlock),
}

pub enum VerifierOutput {
    VerifiedLightBlock(TrustedState),
}

pub enum VerifierError {
    ValidationFailed,
    NotEnoughTrust,
}

pub type VerifierResult = Result<VerifierOutput, VerifierError>;

pub async fn verifier(input: VerifierInput, co: Co<VerifierResult>) {
    let not_enough_trust = rand::random();

    match input {
        VerifierInput::VerifyLightBlock(lb) => {
            if not_enough_trust {
                co.yield_(Err(VerifierError::NotEnoughTrust)).await;
            } else {
                let ts = TrustedState;
                co.yield_(Ok(VerifierOutput::VerifiedLightBlock(ts))).await;
            }
        }
    }
}

pub enum IoInput {
    FetchState,
}

pub enum IoOutput {
    FetchedState,
}

pub enum IoError {
    NotFound,
    Timeout,
}

pub type IoResult = Result<IoOutput, IoError>;

pub async fn io(input: IoInput, co: Co<IoResult>) {
    match input {
        IoInput::FetchState => {
            co.yield_(Ok(IoOutput::FetchedState)).await;
        }
    }
}

pub fn main() {}
