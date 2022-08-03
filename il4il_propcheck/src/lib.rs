//! Property testing library.

pub mod arbitrary;
pub mod assertion;
pub mod generator;
pub mod setup;

pub fn run_property_test<S: setup::Setup, I: arbitrary::Arb, F: Fn(I) -> assertion::Assertion>(test: F) {
    let mut setup = S::default();
    let mut test_count = setup.test_count();
    let mut generator = setup.generator();

    enum Failure {
        Panic(Box<dyn std::any::Any + Send + 'static>),
        Message(&'static str),
    }

    impl Failure {
        fn message(&self) -> &'static str {
            match self {
                Self::Panic(_) => "panic occured",
                Self::Message(message) => message,
            }
        }
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
        eprintln!("test failed with {:?}, {:?}", error.0, error.1.message());
        todo!("handle test shrinking")
    }
}

#[macro_export]
macro_rules! property {
    (fn $test_name:ident<$setup_type:ty>($input_name:ident: $input_type:ty) {
        $test:expr
    }) => {
        #[test]
        fn $test_name() {
            crate::run_property_test::<$setup_type, $input_type, _>(|$input_name| {
                $test
            });
        }
    };

    (fn $test_name:ident($input_name:ident: $input_type:ty) {
        $test:expr
    }) => {
        property! {
            fn $test_name<crate::setup::DefaultSetup>($input_name: $input_type) {
                $test
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assertion;

    property! {
        fn left_shift_equals_multiply_by_two(value: u32) {
            assertion!(value << 1 == value.overflowing_mul(2).0)
        }
    }
}
