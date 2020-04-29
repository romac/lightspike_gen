#![allow(unused_variables, unused_imports)]

use async_recursion::async_recursion;
use genawaiter::{
    rc::{Co, Gen},
    Coroutine, GeneratorState,
};
use std::future::Future;

macro_rules! unwrap {
    ($enum:path, $expr:expr) => {{
        if let $enum(item) = $expr {
            item
        } else {
            unreachable!()
        }
    }};
}

pub fn drain<I, O, E, F>(
    mut gen: Gen<O, I, F>,
    init: I,
    handler: impl Fn(O) -> Result<I, E>,
) -> Result<F::Output, E>
where
    F: Future,
{
    let mut response = init;

    loop {
        match gen.resume_with(response) {
            GeneratorState::Yielded(request) => {
                response = handler(request)?;
            }
            GeneratorState::Complete(result) => return Ok(result),
        }
    }
}

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
}

pub mod demuxer {
    use super::*;
    use {io::*, scheduler::*, verifier::*};

    // pub enum DemuxerInput {
    //     VerifyHeight(Height),
    // }

    // pub enum DemuxerOutput {
    //     Trusted(Vec<LightBlock>),
    // }

    #[derive(Debug)]
    pub enum DemuxerError {
        Scheduler(SchedulerError),
        Verifier(VerifierError),
        Io(IoError),
    }

    // FIXME: This should be a method of a Demuxer struct, and state one of its fields
    pub fn verify_height(
        state: &mut State,
        height: Height,
    ) -> Result<Vec<LightBlock>, DemuxerError> {
        let input = SchedulerInput::VerifyHeight(height);
        let scheduler = Gen::new(|co| scheduler::process(state, input, co));

        let result = drain(scheduler, SchedulerResponse::Init, |req| {
            handle_request(state, req)
        })?;

        let result = result.map_err(|e| DemuxerError::Scheduler(e))?;
        match result {
            SchedulerOutput::TrustedStates(trusted_states) => Ok(trusted_states),
        }
    }

    // FIXME: This should be a method of a Demuxer struct, and state one of its fields
    pub fn verify_light_block(
        state: &State,
        lb: LightBlock,
    ) -> Result<Vec<LightBlock>, DemuxerError> {
        let input = SchedulerInput::VerifyLightBlock(lb);
        let scheduler = Gen::new(|co| scheduler::process(state, input, co));

        let result = drain(scheduler, SchedulerResponse::Init, |req| {
            handle_request(state, req)
        })?;

        let result = result.map_err(|e| DemuxerError::Scheduler(e))?;
        match result {
            SchedulerOutput::TrustedStates(trusted_states) => Ok(trusted_states),
        }
    }

    // FIXME: This should be a method of a Demuxer struct, and state one of its fields
    fn validate_light_block(state: &State, lb: LightBlock) -> Result<LightBlock, DemuxerError> {
        let input = VerifierInput::VerifyLightBlock(lb);
        let result = verifier::process(input).map_err(|e| DemuxerError::Verifier(e))?;
        match result {
            VerifierOutput::VerifiedLightBlock(lb) => Ok(lb),
        }
    }

    // FIXME: This should be a method of a Demuxer struct
    pub fn fetch_light_block(height: Height) -> Result<LightBlock, DemuxerError> {
        let input = IoInput::FetchLightBlock(height);
        let result = io::process(input).map_err(|e| DemuxerError::Io(e))?;
        match result {
            IoOutput::FetchedLightBlock(lb) => Ok(lb),
        }
    }

    fn handle_request(
        state: &State,
        request: SchedulerRequest,
    ) -> Result<SchedulerResponse, DemuxerError> {
        match request {
            SchedulerRequest::GetLightBlock(height) => {
                fetch_light_block(height).map(|lb| SchedulerResponse::LightBlock(lb))
            }
            SchedulerRequest::VerifyLightBlock(lb) => match verify_light_block(state, lb) {
                Ok(ts) => Ok(SchedulerResponse::Verified(Ok(ts))),
                Err(DemuxerError::Verifier(err)) => Ok(SchedulerResponse::Verified(Err(err))),
                Err(err) => Err(err),
            },
            SchedulerRequest::ValidateLightBlock(lb) => match validate_light_block(state, lb) {
                Ok(ts) => Ok(SchedulerResponse::Validated(Ok(ts))),
                Err(DemuxerError::Verifier(err)) => Ok(SchedulerResponse::Validated(Err(err))),
                Err(err) => Err(err),
            },
        }
    }
}

pub mod scheduler {
    use super::io::IoError;
    use super::verifier::VerifierError;
    use super::*;

    pub enum SchedulerInput {
        VerifyHeight(Height),
        VerifyLightBlock(LightBlock),
    }

    pub enum SchedulerOutput {
        TrustedStates(Vec<LightBlock>),
    }

    #[derive(Debug)]
    pub enum SchedulerError {
        InvalidLightBlock(LightBlock, VerifierError),
    }

    pub enum SchedulerRequest {
        GetLightBlock(Height),
        VerifyLightBlock(LightBlock),
        ValidateLightBlock(LightBlock),
    }

    pub enum SchedulerResponse {
        Init,
        LightBlock(LightBlock),
        Validated(Result<LightBlock, VerifierError>),
        Verified(Result<Vec<LightBlock>, VerifierError>),
    }

    pub type SchedulerResult = Result<SchedulerOutput, SchedulerError>;

    pub async fn process(
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

    #[derive(Debug)]
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
        FetchLightBlock(Height),
    }

    pub enum IoOutput {
        FetchedLightBlock(LightBlock),
    }

    #[derive(Debug)]
    pub enum IoError {
        NotFound,
        Timeout,
    }

    pub type IoResult = Result<IoOutput, IoError>;

    pub fn process(input: IoInput) -> IoResult {
        match input {
            IoInput::FetchLightBlock(height) => Ok(IoOutput::FetchedLightBlock(LightBlock)),
        }
    }
}

pub fn main() {
    let mut state = State;
    let result = demuxer::verify_height(&mut state, 42);
    dbg!(result);
}

