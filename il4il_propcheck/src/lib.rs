//! Property testing library.

mod assertion;

pub mod arbitrary;
pub mod generator;
pub mod setup;
pub mod test;

pub use arbitrary::Arb;
pub use assertion::Assertion;

pub fn run_property_test<S: setup::Setup, T: test::PropertyTest>(test: T) {
    let mut arguments_buffer = String::new();

    let mut setup = S::default();
    let expected_test_count = setup.test_count();
    let mut actual_test_count = 0;
    let mut generator = setup.generator();

    let failure = loop {
        if actual_test_count == expected_test_count {
            break Ok(());
        }

        match test.clone().test(&mut arguments_buffer, &mut generator) {
            Ok(test::NonFailure::Success) => actual_test_count += 1,
            Ok(test::NonFailure::Skipped) => (),
            Err((shrinker, bad)) => break Err((arguments_buffer.clone(), shrinker, bad)),
        }
    };

    if let Err((initial_inputs, shrinker, bad)) = failure {
        eprintln!("Test failed with ({initial_inputs}), {bad}");
        let mut failure_count = 1;

        for shrunk_test in shrinker {
            match test::ShrunkTest::run(shrunk_test, &mut arguments_buffer) {
                Ok(_) => (),
                Err(bad) => {
                    eprintln!("> Test failed with ({arguments_buffer}), {bad}");
                    failure_count += 1;
                }
            }
        }

        // TODO: Shrink and print the last_error.
        // TODO: If failure is a panic, then print the whole panic by doing resume_unwind.
        //let last_error = (initial_inputs, bad);
        //for test in shrinker { }

        panic!("Test failed: {actual_test_count} passed, {failure_count} failed")
    } else if actual_test_count < expected_test_count {
        panic!(
            "Unable to generate {expected_test_count} tests, {actual_test_count} passed but {} discarded",
            expected_test_count - actual_test_count
        );
    }
}

#[macro_export]
macro_rules! skip {
    () => {
        return Option<$crate::Assertion>::None;
    };
}

#[macro_export]
macro_rules! property {
    (fn $test_name:ident<$setup_type:ty>($first_input_name:ident: $first_input_type:ty$(, $next_input_name:ident: $next_input_type:ty)*) $test:block) => {
        #[test]
        fn $test_name() {
            fn inner($first_input_name: $first_input_type, $(, $next_input_name: $next_input_type)*) -> Option<$crate::Assertion> {
                $test
            }

            $crate::run_property_test::<$setup_type, _>(inner as fn($first_input_type, $($next_input_type)*) -> _);
        }
    };

    (fn $test_name:ident($first_input_name:ident: $first_input_type:ty$(, $next_input_name:ident: $next_input_type:ty)*) $test:block) => {
        $crate::property!(fn $test_name<$crate::setup::DefaultSetup>($first_input_name: $first_input_type$(, $next_input_name: $next_input_type)*) $test);
    };
}

#[macro_export]
macro_rules! assertion {
    ($e:expr) => {
        Option::<$crate::Assertion>::from(if $e {
            $crate::Assertion::Success
        } else {
            $crate::Assertion::Failure(concat!("assertion failed: ", stringify!($e)).into())
        })
    };
}

#[macro_export]
macro_rules! assertion_eq {
    ($expected:expr, $actual:expr) => {
        Option::<$crate::Assertion>::from($crate::Assertion::are_equal($expected, $actual))
    };
}

#[cfg(test)]
mod tests {
    use crate::*;

    property! {
        fn left_shift_equals_multiply_by_two(value: u32) {
            assertion!(value << 1 == value.overflowing_mul(2).0)
        }
    }
}
