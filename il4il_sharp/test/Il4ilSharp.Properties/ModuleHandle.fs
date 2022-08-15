module Il4ilSharp.Properties.ModuleHandle

open Il4ilSharp.Interop

open Expecto

[<Tests>]
let tests =
    testList "module handle" [
        testCase "module with name is valid" <| fun() ->
            let mdle = new ModuleHandle() // No Dispose call is intended here
            let name = new IdentifierHandle("MyModuleName")
            mdle.AddMetadataName name
            let validated = Il4ilSharp.ValidModule mdle
            let name' = validated.Metadata[0] :?> Il4ilSharp.ModuleNameMetadata
            Expect.equal (name'.Name.ToString()) "MyModuleName" "module names should match"
    ]
