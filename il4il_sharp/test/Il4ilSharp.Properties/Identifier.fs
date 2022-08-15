module Il4ilSharp.Properties.Identifier

open Il4ilSharp.Interop

open Expecto

[<Tests>]
let tests =
    testList "identifier" [
        testProperty "handle from string returns original string" <| fun (s: string) ->
            // TODO: Figure out how to properly skip some tests
            if not(System.String.IsNullOrEmpty s) then
                use handle = new IdentifierHandle(s)
                Expect.equal (handle.ToString()) s "strings should be the same"
    ]
