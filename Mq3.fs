namespace CurrencyOne.Mq3

open System

type Trace(routingKey: string, replyTo: string, correlactionId: Guid, tracePoints: Guid array) =
    member this.RoutingKey = routingKey
    member this.ReplyTo = replyTo
    member this.CorrelationId = correlactionId
    member this.TracePoints = tracePoints
    member this.Next() =
        let newTracePoints = Array.create (tracePoints.Length + 1) (Guid.NewGuid())
        Array.Copy(tracePoints, newTracePoints, tracePoints.Length)
        Trace(routingKey, replyTo, correlactionId, tracePoints)

type Body = byte []

type MQConsumer = Body -> Trace -> Async<unit>

type IMQueue =
    abstract Bind: routingKey:string * MQConsumer -> IMQueue
    abstract Done: unit -> unit


type IMq3 =
    abstract EventQueue: qname:string -> prefetchCount:int -> IMQueue
    abstract QueryQueue: qname:string -> IMQueue

    abstract Publish: routingKey:string -> Body -> Trace -> Trace
    abstract Query: routingKey:string -> Body -> Trace -> Async<Body>
    abstract PublishResponse: Trace -> Body -> unit
