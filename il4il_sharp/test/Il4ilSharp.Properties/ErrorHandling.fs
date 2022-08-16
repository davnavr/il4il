module Il4ilSharp.Properties.ErrorHandling

open Il4ilSharp.Interop

open Expecto

[<Tests>]
let tests =
    testList "error handling" [
        testProperty "handle from string returns original string" <| fun (msg: string) ->
            let expected = if msg = null then System.String.Empty else msg
            use handle = new ErrorHandle(expected)
            Expect.equal (handle.ToString()) expected "error messages should be the same"
    ]
