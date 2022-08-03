//! Property testing library.

pub mod arbitrary;
pub mod assertion;
pub mod generator;
pub mod setup;

fn test_runner<S: setup::Setup, I: arbitrary::Arb, F: Fn(I) -> assertion::Assertion>(test: F) {

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
