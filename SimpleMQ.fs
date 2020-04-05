namespace SimpleMQ

open System

type Trace internal (routingKey: string, replyTo: string, correlationId: Guid, tracePoints: Guid array) =
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

type MQueue =
    abstract Bind: routingKey:string * MQConsumer -> MQueue
    abstract Done: unit -> unit

type SimpleMQ =
    abstract EventQueue: name:string * prefetchCount:int -> MQueue
    abstract QueryQueue: name:string -> MQueue

    abstract PublishQuery: Trace * routingKey:string * Body * ?contentType:string -> Async<Body>
    abstract PublishEvent: Trace * routingKey:string * Body * ?contentType:string -> Trace
    abstract PublishResponse: Trace * Body * ?contentType:string -> unit

type private BindingKeyPattern =
    | Exact of string
    | StartsWith of string

type internal Binding(bindingKey: string, consumer: MQConsumer) =
    let pattern =
        if bindingKey.EndsWith(".#")
        then StartsWith (bindingKey.Replace(".#", ""))
        else Exact bindingKey

    member val Consumer = consumer with get

    member this.Matches routingKey =
        match pattern with
        | Exact bindingKey -> routingKey = bindingKey
        | StartsWith bindingKey -> routingKey.StartsWith(bindingKey)
