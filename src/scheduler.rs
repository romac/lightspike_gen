use crate::io::IoError;
use crate::prelude::*;
use crate::verifier::VerifierError;

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
