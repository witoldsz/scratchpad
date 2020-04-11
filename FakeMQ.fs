namespace SimpleMQ

type FakeMQ () =
    interface SimpleMQ with
        member this.EventQueue (name:string, prefetchCount, bindings) =
            failwith ("TODO")

        member this.QueryQueue (name:string, prefetchCount, bindings) =
            failwith ("TODO")

        member this.PublishQuery (trace, routingKey, body, contentType) =
            failwith ("TODO")

        member this.PublishEvent (trace, routingKey, body, contentType) =
            failwith ("TODO")

        member this.PublishResponse (trace , body, contentType) =
            failwith ("TODO")