//! Property testing library.

pub mod arbitrary;
pub mod assertion;
pub mod generator;
pub mod setup;

#[macro_export]
macro_rules! property {
    (fn $test_name:ident<$setup_type:ty>($input_type:ty) {
        $test:expr
    }) => {
        #[test]
        fn $test_name {
            todo!()
        }
    };

    (fn $test_name:ident($input_type:ty) {
        $test:expr
    }) => {
        property! {{
            fn $test_name<setup::DefaultSetup>($input_type) {
                $test
            }
        }}
    }
}
