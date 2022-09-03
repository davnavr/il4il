use il4il_vm::runtime;

fn main() {
    let program = il4il_samples::return_int("Ok", 1);
    let runtime = runtime::Runtime::new();
    let module = runtime.load_module(program);
    let a = module.module();
}
