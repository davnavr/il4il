module Il4ilSharp.Properties.Identifier

open Il4ilSharp.Interop

open Expecto

[<Tests>]
let tests =
    testList "identifier" [
        testCase "empty string throws exception" <| fun() ->
            Expect.throwsC
                (fun() -> Il4ilSharp.IdentifierString(System.String.Empty) |> ignore)
                (fun e -> Expect.stringContains (e.ToString()) "empty" "exception should describe empty")

        testProperty "handle from string returns original string" <| fun (s: string) ->
            // TODO: Figure out how to properly skip some tests
            if not(System.String.IsNullOrEmpty s || s.Contains '\u0000') then
                use handle = new IdentifierHandle(s)
                Expect.equal (handle.ToString()) s "strings should be the same"
    ]
