module Main

open RabbitMQ.Client
open SimpleMQ

[<EntryPoint>]
let main argv =
    let mq = RabbitSimpleMQ.connect "my-fsharp-demo" (ConnectionFactory())

    let event_sayHello =
        "event.SayHello", (fun event trace ->
        async {
            printfn "%A" trace
            printfn "Hello to you: %s" event
        })

    mq.EventQueue(name = "events", bindings = [ event_sayHello ])

    async {
        printfn "Publishing query…"
        let! queryResponse = mq.PublishQuery(Trace.Empty, "query.settings", """["buffer-rates"]""")
        printfn "Query response: %s" queryResponse
    }
    |> Async.RunSynchronously

    0
