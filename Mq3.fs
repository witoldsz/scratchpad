namespace CurrencyOne.Infra

open System

module MQ3 =
    type Trace(routingKey: string, replyTo: string, correlationId: Guid, tracePoints: Guid array) =
        member this.RoutingKey = routingKey
        member this.ReplyTo = replyTo
        member this.CorrelationId = correlationId
        member this.TracePoints = tracePoints

        static member Empty = Trace("", "", Guid.Empty, Array.empty)

        member internal this.Next() =
            let newTracePoints = Array.create (tracePoints.Length + 1) (Guid.NewGuid())
            Array.Copy(tracePoints, newTracePoints, tracePoints.Length)
            Trace(routingKey, replyTo, correlationId, tracePoints)

    type Body = string

    type MQConsumer = Body -> Trace -> Async<unit>

    type IMQueue =
        abstract Bind: routingKey:string * MQConsumer -> IMQueue
        abstract Done: unit -> unit

    type IMQ3 =
        abstract EventQueue: qname:string -> prefetchCount:int -> IMQueue
        abstract QueryQueue: qname:string -> IMQueue

        abstract Publish: routingKey:string -> Body -> Trace -> Trace
        abstract Query: routingKey:string -> Body -> Trace -> Async<Body>
        abstract PublishResponse: Trace -> Body -> unit

    type private BindingKeyPattern =
        | Exact of string
        | StartsWith of string

    type internal Binding(bindingKey: string, consumer: MQConsumer) =
        let pattern =
            if bindingKey.EndsWith(".#")
            then StartsWith (bindingKey.Replace(".#", ""))
            else Exact bindingKey

        member val Consumer = consumer with get

        member this.Matches (routingKey: string) =
            match pattern with
            | Exact bindingKey -> routingKey = bindingKey
            | StartsWith bindingKey -> routingKey.StartsWith(bindingKey)
