#[cfg(test)]
mod tests {
    use genawaiter::{rc::Gen, GeneratorState};

    #[test]
    fn it_works() {
        let mut gen = Gen::new(|co| async move {
            let mut num = 0_u32;
            while let Some(add) = co.yield_(num).await {
                num += add;
            }
        });

        // Note: The first resume argument will be lost.
        // This is because at the time the first value is sent,
        // there is no future being awaited inside the generator,
        // so there is no place the value could go where the generator could observe it.
        assert_eq!(gen.resume_with(None), GeneratorState::Yielded(0));

        assert_eq!(gen.resume_with(Some(1)), GeneratorState::Yielded(1));
        assert_eq!(gen.resume_with(Some(2)), GeneratorState::Yielded(3));
        assert_eq!(gen.resume_with(Some(3)), GeneratorState::Yielded(6));
    }
}
