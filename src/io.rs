use crate::prelude::*;

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

pub type Io = Box<dyn Fn(IoInput) -> IoResult>;

pub fn handler<F>(f: F) -> Io
where
    F: Fn(IoInput) -> IoResult + 'static,
{
    Box::new(f)
}

