#![allow(unused_variables, unused_imports)]

use async_recursion::async_recursion;
use genawaiter::{
    rc::{Co, Gen},
    GeneratorState,
};

macro_rules! unwrap {
    ($enum:path, $expr:expr) => {{
        if let $enum(item) = $expr {
            item
        } else {
            unreachable!()
        }
    }};
}

pub type Height = u64;

#[derive(Copy, Clone)]
pub struct LightBlock;
#[derive(Copy, Clone)]
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
        TrustedStates(Vec<LightBlock>),
    }

    pub enum SchedulerError {
        InvalidLightBlock(LightBlock, VerifierError),
    }

    pub enum SchedulerRequest {
        GetLightBlock(Height),
        VerifyLightBlock(LightBlock),
        ValidateLightBlock(LightBlock),
    }

    pub enum SchedulerResponse {
        LightBlock(LightBlock),
        Validated(Result<LightBlock, VerifierError>),
        Verified(Result<Vec<LightBlock>, VerifierError>),
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
        if let Some(trusted_state) = state.get_trusted_state(height) {
            Ok(SchedulerOutput::TrustedStates(vec![trusted_state]))
        } else {
            let response = co.yield_(SchedulerRequest::GetLightBlock(height)).await;
            let lb = unwrap!(SchedulerResponse::LightBlock, response);

            verify_light_block(state, lb, co).await
        }
    }

    pub async fn verify_light_block(
        state: &State,
        lb: LightBlock,
        co: Co<SchedulerRequest, SchedulerResponse>,
    ) -> SchedulerResult {
        let response = co.yield_(SchedulerRequest::ValidateLightBlock(lb)).await;

        let result = unwrap!(SchedulerResponse::Validated, response);
        match result {
            Err(VerifierError::NotEnoughTrust) => do_bisection(state, lb, co).await,
            Err(err) => Err(SchedulerError::InvalidLightBlock(lb, err)),
            Ok(trusted_state) => Ok(SchedulerOutput::TrustedStates(vec![trusted_state])),
        }
    }

    pub async fn do_bisection(
        state: &State,
        lb: LightBlock,
        co: Co<SchedulerRequest, SchedulerResponse>,
    ) -> SchedulerResult {
        let pivot_height = rand::random();

        let pivot_lb = co
            .yield_(SchedulerRequest::GetLightBlock(pivot_height))
            .await;

        let pivot_lb = unwrap!(SchedulerResponse::LightBlock, pivot_lb);

        let pivot_response = co
            .yield_(SchedulerRequest::VerifyLightBlock(pivot_lb))
            .await;

        let mut trusted_states = unwrap!(SchedulerResponse::Verified, pivot_response)
            .map_err(|e| SchedulerError::InvalidLightBlock(pivot_lb, e))?;

        let lb_response = co.yield_(SchedulerRequest::ValidateLightBlock(lb)).await;
        let trusted_state = unwrap!(SchedulerResponse::Validated, lb_response)
            .map_err(|e| SchedulerError::InvalidLightBlock(lb, e))?;

        trusted_states.push(trusted_state);

        Ok(SchedulerOutput::TrustedStates(trusted_states))
    }
}

pub mod verifier {
    use super::*;

    pub enum VerifierInput {
        VerifyLightBlock(LightBlock),
    }

    pub enum VerifierOutput {
        VerifiedLightBlock(LightBlock),
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
                    let lb = LightBlock;
                    Ok(VerifierOutput::VerifiedLightBlock(lb))
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
