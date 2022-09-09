use il4il_vm::runtime;

fn main() {
    let program = il4il_samples::return_int("Ok", 1);
    let runtime = runtime::Runtime::new();
    let module = runtime.load_module(program, None);
    let mut interpreter = module
        .interpret_entry_point(Default::default())
        .unwrap()
        .expect("entry point should exist");
    let return_values = loop {
        if let Some(values) = interpreter.step().unwrap() {
            break values;
        }
    };

    assert_eq!(
        il4il_vm::interpreter::value::Value::into_u32(return_values[0].clone(), runtime.configuration().endianness),
        1u32
    );
}
