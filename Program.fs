module Main

open RabbitMQ.Client
open SimpleMQ

[<EntryPoint>]
let main argv =
    Async.RunSynchronously (async {
        let mq = RabbitSimpleMQ.connect "my-fsharp-demo" (ConnectionFactory())

        let event_SayHello event (trace: Trace) = async {
            printfn "%A" trace
            printfn "Hello to you: %s" event
        }

        let eventsQueue = mq.EventQueue(name = "events", prefetchCount = 0)

        eventsQueue
            .Bind("event.sayHello", event_SayHello)
            .Done()

        printfn "Publishing query…"
        let! queryResponse = mq.PublishQuery(Trace.Empty, "query.settings", """["buffer-rates"]""")
        printfn "Query response: %s" queryResponse
    })
    0
