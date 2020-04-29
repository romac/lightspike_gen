use crate::prelude::*;
use crate::{io::*, scheduler::*, verifier::*};

#[derive(Debug)]
pub enum DemuxerError {
    Scheduler(SchedulerError),
    Verifier(VerifierError),
    Io(IoError),
}

// TODO: Introducer Demuxer structs holding state + references to components.

fn run_scheduler(
    state: &mut State,
    input: SchedulerInput,
) -> Result<SchedulerOutput, DemuxerError> {
    let scheduler = Gen::new(|co| scheduler::process(state.trusted_store_reader(), input, co));

    let result = drain(scheduler, SchedulerResponse::Init, move |req| {
        handle_request(state, req)
    })?;

    result.map_err(|e| DemuxerError::Scheduler(e))
}

pub fn verify_height(state: &mut State, height: Height) -> Result<Vec<LightBlock>, DemuxerError> {
    let input = SchedulerInput::VerifyHeight(height);
    let result = run_scheduler(state, input)?;

    match result {
        SchedulerOutput::TrustedStates(trusted_states) => {
            state.add_trusted_states(trusted_states.clone());
            Ok(trusted_states)
        }
    }
}

// FIXME: This should be a method of a Demuxer struct, and state one of its fields
pub fn verify_light_block(
    state: &mut State,
    lb: LightBlock,
) -> Result<Vec<LightBlock>, DemuxerError> {
    let input = SchedulerInput::VerifyLightBlock(lb);
    let result = run_scheduler(state, input)?;

    match result {
        SchedulerOutput::TrustedStates(trusted_states) => {
            state.add_trusted_states(trusted_states.clone());
            Ok(trusted_states)
        }
    }
}

// FIXME: This should be a method of a Demuxer struct, and state one of its fields
fn validate_light_block(state: &mut State, lb: LightBlock) -> Result<LightBlock, DemuxerError> {
    let input = VerifierInput::VerifyLightBlock(lb);
    let result = verifier::process(input).map_err(|e| DemuxerError::Verifier(e))?;
    match result {
        VerifierOutput::VerifiedLightBlock(lb) => {
            state.add_valid_light_block(lb.clone());
            Ok(lb)
        }
    }
}

// FIXME: This should be a method of a Demuxer struct
pub fn fetch_light_block(state: &mut State, height: Height) -> Result<LightBlock, DemuxerError> {
    let input = IoInput::FetchLightBlock(height);
    let result = io::process(input).map_err(|e| DemuxerError::Io(e))?;
    match result {
        IoOutput::FetchedLightBlock(lb) => {
            state.add_fetched_light_block(lb.clone());
            Ok(lb)
        }
    }
}

fn handle_request(
    state: &mut State,
    request: SchedulerRequest,
) -> Result<SchedulerResponse, DemuxerError> {
    match request {
        SchedulerRequest::GetLightBlock(height) => {
            fetch_light_block(state, height).map(|lb| SchedulerResponse::LightBlock(lb))
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
