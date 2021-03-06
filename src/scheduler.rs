use crate::io::IoError;
use crate::prelude::*;
use crate::verifier::VerifierError;
use std::{future::Future, pin::Pin};

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
    store: TSReader,
    input: SchedulerInput,
    co: Co<SchedulerRequest, SchedulerResponse>,
) -> SchedulerResult {
    match input {
        SchedulerInput::VerifyHeight(height) => verify_height(store, height, co).await,
        SchedulerInput::VerifyLightBlock(lb) => verify_light_block(store, lb, co).await,
    }
}

pub async fn verify_height(
    store: TSReader,
    height: Height,
    co: Co<SchedulerRequest, SchedulerResponse>,
) -> SchedulerResult {
    if let Some(trusted_state) = store.get(height) {
        Ok(SchedulerOutput::TrustedStates(vec![trusted_state]))
    } else {
        let response = co.yield_(SchedulerRequest::GetLightBlock(height)).await;
        let lb = unwrap!(SchedulerResponse::LightBlock, response);

        verify_light_block(store, lb, co).await
    }
}

pub async fn verify_light_block(
    store: TSReader,
    lb: LightBlock,
    co: Co<SchedulerRequest, SchedulerResponse>,
) -> SchedulerResult {
    let response = co.yield_(SchedulerRequest::ValidateLightBlock(lb)).await;

    let result = unwrap!(SchedulerResponse::Validated, response);
    match result {
        Err(VerifierError::NotEnoughTrust) => do_bisection(store, lb, co).await,
        Err(err) => Err(SchedulerError::InvalidLightBlock(lb, err)),
        Ok(trusted_state) => Ok(SchedulerOutput::TrustedStates(vec![trusted_state])),
    }
}

pub async fn do_bisection(
    store: TSReader,
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

pub type Scheduler = Box<
    dyn Fn(
        TSReader,
        SchedulerInput,
        Co<SchedulerRequest, SchedulerResponse>,
    ) -> Pin<Box<dyn Future<Output = SchedulerResult>>>,
>;

pub fn handler<F0, F>(f: F0) -> Scheduler
where
    F0: Fn(TSReader, SchedulerInput, Co<SchedulerRequest, SchedulerResponse>) -> F + 'static,
    F: Future<Output = SchedulerResult> + 'static,
{
    Box::new(move |s, i, c| Box::pin(f(s, i, c)))
}

