module Il4ilSharp.Properties.Identifier

open Il4ilSharp.Interop

open Expecto

[<Tests>]
let tests =
    testList "identifier" [
        testProperty "handle from string returns original string" <| fun (s: string) ->
            use handle = new IdentifierHandle(s)
            Expect.equal (handle.ToString()) s "strings should be the same"
    ]
