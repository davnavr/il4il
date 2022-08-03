//! Property testing library.

pub mod arbitrary;
pub mod assertion;
pub mod generator;
pub mod setup;

fn test_runner<S: setup::Setup, I: arbitrary::Arb, F: Fn(I) -> assertion::Assertion>(test: F) {
    let mut setup = S::default();
    let mut test_count = setup.test_count();
    let mut generator = setup.generator();

    enum Failure {
        Panic(Box<dyn std::any::Any + Send + 'static>),
        Message(&'static str),
    }

    fn run_test<I, F, R>(gen: &mut generator::Gen<'_, R>, test: &F) -> Result<(), (I, Failure)> where I: arbitrary::Arb, F: Fn(I) -> assertion::Assertion, R: rand::Rng + ?Sized {
        let value = I::arbitrary(gen);
        let assertion = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            test(value.clone())
        }));

        match assertion {
            Ok(assertion::Assertion::Success) => Ok(()),
            Ok(assertion::Assertion::Failure(message)) => Err((value, Failure::Message(message))),
            Err(panic_info) => Err((value, Failure::Panic(panic_info))),
        }
    }

    let failure = loop {
        if test_count == 0 {
            break Ok(());
        }

        match run_test(&mut generator, &test) {
            Ok(()) => test_count -= 1,
            Err(error) => break Err(error),
        }
    };

    if let Err(error) = failure {
        
        todo!("handle test failure")
    }
}

#[macro_export]
macro_rules! property {
    (fn $test_name:ident<$setup_type:ty>($input_name:ident: $input_type:ty) {
        $test:expr
    }) => {
        #[test]
        fn $test_name() {
            todo!()
        }
    };

    (fn $test_name:ident($input_name:ident: $input_type:ty) {
        $test:expr
    }) => {
        property! {
            fn $test_name<setup::DefaultSetup>($input_name: $input_type) {
                $test
            }
        }
    }
}

#[cfg(test)]
mod tests {
    property! {
        fn left_shift_equals_multiply_by_two(value: u32) {
            assertion::assertion!(value << 1 == value.overflowing_mul(2).0)
        }
    }
}
