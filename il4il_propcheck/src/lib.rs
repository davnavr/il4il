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

        for test in shrinker {
            match test::ShrunkTest::run(test, &mut arguments_buffer) {
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

// pub fn run_property_test<S: setup::Setup, T: test::Test>(test: T) {
//     enum Failure {
//         Panic(Box<dyn std::any::Any + Send + 'static>),
//         Message(std::borrow::Cow<'static, str>),
//     }

//     impl Failure {
//         fn message(&self) -> &str {
//             match self {
//                 Self::Panic(_) => "panic occured",
//                 Self::Message(message) => message,
//             }
//         }
//     }

//     let mut setup = S::default();
//     let mut test_count = setup.test_count();
//     let mut arguments_buffer = String::new();
//     let mut generator = setup.generator();

//     let failure = loop {
//         if test_count == 0 {
//             break Ok(());
//         }

//         match test.run(&mut arguments_buffer, &mut generator) {
//             Ok(()) => test_count -= 1,
//             Err(error) => break Err(error),
//         }
//     };

//     if let Err(error) = failure {
//         eprintln!("test failed with {:?}, {:?}", error.0, error.1.message());

//         let mut smallest = None;
//         let mut shrunk_count = 0usize;
//         for shrunk in error.0.shrink() {
//             shrunk_count += 1;

//             match run_test(shrunk, &test) {
//                 Err(f)
//                     if match smallest {
//                         None => true,
//                         Some((ref small, _)) => arbitrary::Comparable::is_smaller_than(small, &f.0),
//                     } =>
//                 {
//                     smallest = Some(f)
//                 }
//                 _ => (),
//             }
//         }

//         let message = if let Some((shrunk, f)) = smallest {
//             eprintln!("shrunk {} times down to {:?}", shrunk_count, shrunk);
//             f
//         } else {
//             error.1
//         };

//         match message {
//             Failure::Message(msg) => panic!("{}", msg),
//             Failure::Panic(panic) => std::panic::resume_unwind(panic),
//         }
//     }
// }

#[macro_export]
macro_rules! skip {
    () => {
        return Option<$crate::Assertion>::None;
    };
}

#[macro_export]
macro_rules! property {
    (fn $test_name:ident<$setup_type:ty>($input_name:ident: $input_type:ty) {
        $test:expr
    }) => {
        #[test]
        fn $test_name() {
            //$crate::run_property_test::<$setup_type, $input_type, _>(|$input_name| $test);
            todo!()
        }
    };

    (fn $test_name:ident($input_name:ident: $input_type:ty) {
        $test:expr
    }) => {
        $crate::property! {
            fn $test_name<$crate::setup::DefaultSetup>($input_name: $input_type) {
                $test
            }
        }
    };
}

#[macro_export]
macro_rules! assertion {
    ($e:expr) => {
        Some(if $e {
            $crate::Assertion::Success
        } else {
            $crate::Assertion::Failure(concat!("assertion failed: ", stringify!($e)).into())
        })
    };
}

#[macro_export]
macro_rules! assertion_eq {
    ($expected:expr, $actual:expr) => {
        Some($crate::Assertion::are_equal($expected, $actual))
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
