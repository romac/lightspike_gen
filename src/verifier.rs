use crate::prelude::*;

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
