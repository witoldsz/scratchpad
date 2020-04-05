namespace SimpleMQ

open RabbitMQ.Client
open RabbitMQ.Client.Events
open System
open System.Text
open System.Threading
open Utils

type private QueueProps =
    { name: string
      exchangeName: string
      prefetchCount: int
      durable: bool
      exclusive: bool
      autoDelete: bool
      autoAck: bool }

    static member Query name =
        { name = name
          exchangeName = "amq.topic"
          prefetchCount = 10
          durable = false
          exclusive = true
          autoDelete = true
          autoAck = true }

    static member Event name prefetchCount =
        { name = name
          exchangeName = "amq.topic"
          prefetchCount = prefetchCount
          durable = true
          exclusive = false
          autoDelete = false
          autoAck = false }

type private RabbitMQueue (conn: IConnection, p: QueueProps) =

    let receivingChannel = conn.CreateModel()

    let mutable bindings: Binding list = List.empty

    let onEvent (event: BasicDeliverEventArgs) =
        let _, traceHeader = event.BasicProperties.Headers.TryGetValue("trace")
        let tracePoints =
            match traceHeader with
            | :? Collections.Generic.List<obj> as list ->
                list.ConvertAll(fun it -> Guid.Parse(Encoding.UTF8.GetString(it :?> byte[]))).ToArray()
            | _ -> Array.empty

        let routingKey = event.RoutingKey
        let trace =
            Trace(
                routingKey = routingKey,
                replyTo = event.BasicProperties.ReplyTo,
                correlationId = Guid.Parse(event.BasicProperties.CorrelationId),
                tracePoints = tracePoints)

        let body = Encoding.UTF8.GetString(event.Body)
        Async.Start (async {
            do! bindings
                |> List.tryFind (fun b -> b.Matches routingKey)
                |> Option.map (fun b -> b.Consumer body trace)
                |> Option.defaultValue (async { return () })

            if p.autoAck
            then receivingChannel.BasicAck(deliveryTag = event.DeliveryTag, multiple = false)
        })

    do
        logInfo "MQ: declare queue [%s]" p.name
        receivingChannel.BasicQos(prefetchSize = 0u, prefetchCount = uint16 p.prefetchCount, ``global`` = true)
        receivingChannel.QueueDeclare
            (queue = p.name, durable = p.durable, exclusive = p.exclusive, autoDelete = p.autoDelete)
        |> ignore

    interface MQueue with

        member this.Bind(bindingKey, consumer) =
            logInfo "MQ: binding queue [%s] to <%s>" p.name bindingKey
            receivingChannel.QueueBind(p.name, p.exchangeName, bindingKey)
            bindings <- Binding(bindingKey, consumer) :: bindings
            this :> MQueue

        member this.Done() =
            let eventConsumer = EventingBasicConsumer(receivingChannel)
            eventConsumer.Received.Add onEvent
            receivingChannel.BasicConsume(queue = p.name, autoAck = p.autoAck, consumer = eventConsumer) |> ignore
            logInfo "MQ: binding queue [%s] done" p.name

type private RabbitSimpleMQ (moduleName: string, cf: ConnectionFactory) as this =

    let mq = this :> SimpleMQ
    let conn = cf.CreateConnection()
    let publishChannel = conn.CreateModel()
    let queries = Collections.Concurrent.ConcurrentDictionary<Guid, Body -> unit>()

    let queue (p: QueueProps) =
        RabbitMQueue(conn, p) :> MQueue

    let createPublishProps (trace: Trace) (contentType: string option) =
        let props = publishChannel.CreateBasicProperties()
        props.Headers <- Map.ofList [
            "trace", trace.TracePoints :> obj
            "publisher", moduleName :> obj
            "ts", DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ss.fffZ") :> obj
        ]
        props.ContentType <- Option.defaultValue "application/json" contentType
        props

    let publish exchange routingKey props (body: string) =
        fun () -> publishChannel.BasicPublish(exchange, routingKey, props, Encoding.UTF8.GetBytes body)
        |> lock publishChannel

    do
        mq.QueryQueue(moduleName)
            .Bind("ping", fun body trace -> async { mq.PublishResponse(trace, moduleName, "text/plain") })
            .Bind(moduleName, fun body trace -> async {
                return queries
                |> DictUtil.tryRemove(trace.CorrelationId)
                |> Option.map (fun resolve -> resolve body)
                |> Option.defaultValue ()
            })
            .Done()

    interface SimpleMQ with

        member this.QueryQueue name =
            queue (QueueProps.Query name)

        member this.EventQueue (name, prefetchCount) =
            queue (QueueProps.Event name prefetchCount)

        member this.PublishEvent (oldTrace, routingKey, body, contentType) =
            let trace = oldTrace.Next()
            let props = createPublishProps trace contentType
            props.Persistent <- true
            publish "amq.topic" routingKey props body
            trace

        member this.PublishResponse (oldTrace, body, contentType) =
            let trace = oldTrace.Next()
            let props = createPublishProps trace contentType
            props.Persistent <- false
            props.CorrelationId <- trace.CorrelationId.ToString()
            publish "" trace.ReplyTo props body

        member this.PublishQuery (oldTrace, routingKey, body, contentType) =
            let trace = oldTrace.Next()
            let correlationId = Guid.NewGuid()
            let props = createPublishProps trace contentType
            props.ReplyTo <- moduleName
            props.CorrelationId <- correlationId.ToString()
            props.Persistent <- false
            publish "amq.topic" routingKey props body
            Async.FromContinuations (fun (resolve, reject, _) ->
                let timeout = async {
                    do! Async.Sleep 3000
                    queries.TryRemove correlationId |> ignore
                    reject (TimeoutException (sprintf "routing key: %s query body: %O" routingKey body)) // TODO
                }
                let cts = new CancellationTokenSource()
                let resolveAndCancelTimeout result =
                    cts.Cancel()
                    resolve result

                Async.Start (timeout, cts.Token)
                queries.TryAdd(correlationId, resolveAndCancelTimeout) |> ignore
            )

module RabbitSimpleMQ =

    let connect (moduleName:string) (cf: RabbitMQ.Client.ConnectionFactory) =
        RabbitSimpleMQ (moduleName, cf) :> SimpleMQ