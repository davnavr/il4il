# IL4IL

An intermediate language for intermediate languages.

This project has gone through several iterations at this point:

- First was [ubyte](https://github.com/davnavr/ubyte), which was written in [F#](https://fsharp.org/) and provided garbage collection while still being a register-based bytecode in SSA form
- Next was [SAILAR](https://github.com/davnavr/SAILAR-lang/tree/terminators), which was a rewrite in Rust that removed garbage collection and focused on lower level features

This next iteration will aim to provide lower level features in languages such as LLVM IR (e.g. user defined types are just structs) as well as higher level features (e.g. opaque references to refer to GC objects in the destination language)
