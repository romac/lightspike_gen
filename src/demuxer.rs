use crate::prelude::*;
use crate::{io::*, scheduler::*, verifier::*};

#[derive(Debug)]
pub enum DemuxerError {
    Scheduler(SchedulerError),
    Verifier(VerifierError),
    Io(IoError),
}

pub struct Demuxer {
    state: State,
}

impl Demuxer {
    pub fn new(state: State) -> Self {
        Self { state }
    }

    pub fn verify_height(&mut self, height: Height) -> Result<Vec<LightBlock>, DemuxerError> {
        let input = SchedulerInput::VerifyHeight(height);
        let result = self.run_scheduler(input)?;

        match result {
            SchedulerOutput::TrustedStates(trusted_states) => {
                self.state.add_trusted_states(trusted_states.clone());
                Ok(trusted_states)
            }
        }
    }

    pub fn verify_light_block(&mut self, lb: LightBlock) -> Result<Vec<LightBlock>, DemuxerError> {
        let input = SchedulerInput::VerifyLightBlock(lb);
        let result = self.run_scheduler(input)?;

        match result {
            SchedulerOutput::TrustedStates(trusted_states) => {
                self.state.add_trusted_states(trusted_states.clone());
                Ok(trusted_states)
            }
        }
    }

    pub fn validate_light_block(&mut self, lb: LightBlock) -> Result<LightBlock, DemuxerError> {
        let input = VerifierInput::VerifyLightBlock(lb);
        let result = verifier::process(input).map_err(|e| DemuxerError::Verifier(e))?;

        match result {
            VerifierOutput::VerifiedLightBlock(lb) => {
                self.state.add_valid_light_block(lb.clone());
                Ok(lb)
            }
        }
    }

    pub fn fetch_light_block(&mut self, height: Height) -> Result<LightBlock, DemuxerError> {
        let input = IoInput::FetchLightBlock(height);
        let result = io::process(input).map_err(|e| DemuxerError::Io(e))?;
        match result {
            IoOutput::FetchedLightBlock(lb) => {
                self.state.add_fetched_light_block(lb.clone());
                Ok(lb)
            }
        }
    }

    fn handle_request(
        &mut self,
        request: SchedulerRequest,
    ) -> Result<SchedulerResponse, DemuxerError> {
        match request {
            SchedulerRequest::GetLightBlock(height) => self
                .fetch_light_block(height)
                .map(|lb| SchedulerResponse::LightBlock(lb)),

            SchedulerRequest::VerifyLightBlock(lb) => match self.verify_light_block(lb) {
                Ok(ts) => Ok(SchedulerResponse::Verified(Ok(ts))),
                Err(DemuxerError::Verifier(err)) => Ok(SchedulerResponse::Verified(Err(err))),
                Err(err) => Err(err),
            },

            SchedulerRequest::ValidateLightBlock(lb) => match self.validate_light_block(lb) {
                Ok(ts) => Ok(SchedulerResponse::Validated(Ok(ts))),
                Err(DemuxerError::Verifier(err)) => Ok(SchedulerResponse::Validated(Err(err))),
                Err(err) => Err(err),
            },
        }
    }

    pub fn run_scheduler(
        &mut self,
        input: SchedulerInput,
    ) -> Result<SchedulerOutput, DemuxerError> {
        let scheduler =
            Gen::new(|co| scheduler::process(self.state.trusted_store_reader(), input, co));

        let result = drain(scheduler, SchedulerResponse::Init, move |req| {
            self.handle_request(req)
        })?;

        result.map_err(|e| DemuxerError::Scheduler(e))
    }
}
