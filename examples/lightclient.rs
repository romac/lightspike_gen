#![allow(unused_variables, unused_imports)]

use async_recursion::async_recursion;
use genawaiter::{rc::Co, yield_, GeneratorState};

macro_rules! try_result {
    ($expr:expr) => {
        match $expr {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(err) => {
                return ::std::result::Result::Err(err.into());
            }
        }
    };
}

pub type Height = u64;

#[derive(Copy, Clone)]
pub struct LightBlock;
#[derive(Copy, Clone)]
pub struct TrustedState;
#[derive(Copy, Clone)]
pub struct State;

impl State {
    pub fn is_trusted(&self, height: Height) -> bool {
        rand::random()
    }

    pub fn get_trusted_state(&self, height: Height) -> Option<TrustedState> {
        if self.is_trusted(height) {
            Some(TrustedState)
        } else {
            None
        }
    }
}

pub mod demuxer {
    use super::*;
    use super::{io::IoError, scheduler::SchedulerError, verifier::VerifierError};

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
}

pub mod scheduler {
    use super::verifier::VerifierError;
    use super::*;

    pub enum SchedulerInput {
        VerifyHeight(Height),
        VerifyLightBlock(LightBlock),
    }

    pub enum SchedulerOutput {
        TrustedStates(Vec<TrustedState>),
    }

    pub enum SchedulerError {
        InvalidLightBlock(LightBlock),
    }

    pub enum SchedulerRequest {
        GetLightBlock(Height),
        VerifyLightBlock(LightBlock),
    }

    pub enum SchedulerResponse {
        LightBlock(LightBlock),
        Verified(Result<TrustedState, VerifierError>),
    }

    pub type SchedulerResult = Result<SchedulerOutput, SchedulerError>;

    pub async fn scheduler(
        state: &State,
        input: SchedulerInput,
        co: Co<SchedulerRequest, SchedulerResponse>,
    ) -> SchedulerResult {
        match input {
            SchedulerInput::VerifyHeight(height) => verify_height(state, height, co).await,
            SchedulerInput::VerifyLightBlock(lb) => verify_light_block(state, lb, co).await,
        }
    }

    pub async fn verify_height(
        state: &State,
        height: Height,
        co: Co<SchedulerRequest, SchedulerResponse>,
    ) -> SchedulerResult {
        if let Some(ts) = state.get_trusted_state(height) {
            Ok(SchedulerOutput::TrustedStates(vec![ts]))
        } else {
            let response = co.yield_(SchedulerRequest::GetLightBlock(height)).await;
            match response {
                SchedulerResponse::LightBlock(lb) => verify_light_block(state, lb, co).await,
                _ => unreachable!(),
            }
        }
    }

    pub async fn verify_light_block(
        state: &State,
        lb: LightBlock,
        co: Co<SchedulerRequest, SchedulerResponse>,
    ) -> SchedulerResult {
        let response = co.yield_(SchedulerRequest::VerifyLightBlock(lb)).await;

        if let SchedulerResponse::Verified(result) = response {
            match result {
                Ok(ts) => Ok(SchedulerOutput::TrustedStates(vec![ts])),
                Err(VerifierError::Invalid) => Err(SchedulerError::InvalidLightBlock(lb)),
                Err(VerifierError::NotEnoughTrust) => do_bisection(state, lb, co).await,
            }
        } else {
            unreachable!()
        }
    }

    #[async_recursion(?Send)]
    pub async fn do_bisection(
        state: &State,
        lb: LightBlock,
        co: Co<SchedulerRequest, SchedulerResponse>,
    ) -> SchedulerResult {
        let pivot_height = rand::random();

        let pivot_lb = co
            .yield_(SchedulerRequest::GetLightBlock(pivot_height))
            .await;

        match pivot_lb {
            SchedulerResponse::LightBlock(pivot_lb) => {
                let SchedulerOutput::TrustedStates(mut tss) =
                    verify_light_block(state, pivot_lb, co).await?;

                let SchedulerOutput::TrustedStates(mut tss) =
                    verify_light_block(state, pivot_lb, co).await?;

                // let SchedulerOutput::TrustedState(ts) =
                //     try_result!(verify_light_block(state, pivot_lb, co).await);

                todo!()
            }
            _ => unreachable!(),
        }
    }
}

pub mod verifier {
    use super::*;

    pub enum VerifierInput {
        VerifyLightBlock(LightBlock),
    }

    pub enum VerifierOutput {
        VerifiedLightBlock(TrustedState),
    }

    pub enum VerifierError {
        Invalid,
        NotEnoughTrust,
    }

    pub type VerifierResult = Result<VerifierOutput, VerifierError>;

    pub fn process(input: VerifierInput) -> VerifierResult {
        let not_enough_trust = rand::random();

        match input {
            VerifierInput::VerifyLightBlock(_lb) => {
                if not_enough_trust {
                    Err(VerifierError::NotEnoughTrust)
                } else {
                    let ts = TrustedState;
                    Ok(VerifierOutput::VerifiedLightBlock(ts))
                }
            }
        }
    }
}

pub mod io {
    use super::*;

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

    pub fn process(input: IoInput) -> IoResult {
        match input {
            IoInput::FetchState => Ok(IoOutput::FetchedState),
        }
    }
}

pub fn main() {}
