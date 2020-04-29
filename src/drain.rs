use std::future::Future;

use genawaiter::{
    rc::{Co, Gen},
    Coroutine, GeneratorState,
};

pub fn drain<I, O, E, F>(
    mut gen: Gen<O, I, F>,
    init: I,
    mut handler: impl FnMut(O) -> Result<I, E>,
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

