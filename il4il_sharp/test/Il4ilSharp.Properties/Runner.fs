module Il4ilSharp.Properties.Runner

[<EntryPoint>]
let main argv = Expecto.Tests.runTestsInAssemblyWithCLIArgs Seq.empty argv
