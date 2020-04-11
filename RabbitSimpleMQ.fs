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

    static member Query moduleName name prefetchCount =
        { name = moduleName + (if name = "" then "" else ":") + name
          exchangeName = "amq.topic"
          prefetchCount = Option.defaultValue 10 prefetchCount
          durable = false
          exclusive = true
          autoDelete = true
          autoAck = true }

    static member Event moduleName name prefetchCount =
        { name = moduleName + (if name = "" then "" else ":") + name
          exchangeName = "amq.topic"
          prefetchCount = Option.defaultValue 1 prefetchCount
          durable = true
          exclusive = false
          autoDelete = false
          autoAck = false }

type private RabbitSimpleMQ (moduleName: string, cf: ConnectionFactory) =

    let conn = cf.CreateConnection()
    let publishChannel = conn.CreateModel()

    let onEvent (p: QueueProps) (bindings: Binding list) (receivingChannel: IModel) (event: BasicDeliverEventArgs) =
        let _, traceHeader = event.BasicProperties.Headers.TryGetValue("trace")
        let tracePoints =
            match traceHeader with
            | :? Collections.Generic.List<obj> as list ->
                list.ConvertAll(fun it -> Guid.Parse(Encoding.UTF8.GetString(it :?> byte[]))).ToArray()
            | _ ->
                Array.empty

        let routingKey = event.RoutingKey
        let correlationId =
            if event.BasicProperties.CorrelationId = null
            then Guid.Empty
            else Guid.Parse(event.BasicProperties.CorrelationId)

        let trace =
            { routingKey = routingKey
              replyTo = event.BasicProperties.ReplyTo
              correlationId = correlationId
              tracePoints = tracePoints }

        let body = Encoding.UTF8.GetString(event.Body)

        Async.Start (async {
            do! bindings
                |> List.tryFind (fun b -> b.Matches routingKey)
                |> Option.map (fun b -> b.Consumer body trace)
                |> Option.defaultValue (async { return () })

            if not p.autoAck
            then receivingChannel.BasicAck(deliveryTag = event.DeliveryTag, multiple = false)
        })

    let queue (p: QueueProps) (bindings: MQBinding list) =
        logInfo "MQ: declare queue [%s]" p.name
        let receivingChannel = conn.CreateModel()
        receivingChannel.BasicQos(prefetchSize = 0u, prefetchCount = uint16 p.prefetchCount, ``global`` = true)
        receivingChannel.QueueDeclare
            (queue = p.name, durable = p.durable, exclusive = p.exclusive, autoDelete = p.autoDelete)
        |> ignore

        let bindQueue (key, _) =
            logInfo "MQ: binding queue [%s] to <%s>" p.name key
            receivingChannel.QueueBind(p.name, p.exchangeName, key)

        List.iter bindQueue bindings

        let eventConsumer = EventingBasicConsumer(receivingChannel)
        eventConsumer.Received.Add (fun b -> ())
        eventConsumer.Received.Add (onEvent p (List.map Binding bindings) receivingChannel)
        eventConsumer.Shutdown.Add Console.WriteLine
        receivingChannel.BasicConsume(queue = p.name, autoAck = p.autoAck, consumer = eventConsumer) |> ignore
        logInfo "MQ: binding queue [%s] done" p.name

    let createPublishProps (trace: Trace) (contentType: string option) =
        let props = publishChannel.CreateBasicProperties()
        props.Headers <- Map.ofList [
            "trace", trace.tracePoints |> Array.map (fun t -> t.ToString()) :> obj
            "publisher", moduleName :> obj
            "ts", DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ss.fffZ") :> obj
        ]
        props.ContentType <- Option.defaultValue "application/json" contentType
        props

    let publish exchange routingKey props (body: string) =
        let action () = publishChannel.BasicPublish(exchange, routingKey, props, Encoding.UTF8.GetBytes body)
        lock publishChannel action

    member val Queries = Collections.Concurrent.ConcurrentDictionary<Guid, Body -> unit>() with get

    interface SimpleMQ with

        member this.QueryQueue (name, prefetchCount, bindings) =
            queue (QueueProps.Query moduleName name prefetchCount) bindings

        member this.EventQueue (name, prefetchCount, bindings) =
            queue (QueueProps.Event moduleName name prefetchCount) bindings

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
            props.CorrelationId <- trace.correlationId.ToString()
            publish "" trace.replyTo props body

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
                    this.Queries.TryRemove correlationId |> ignore
                    reject (TimeoutException (sprintf "routing key: %s query body: %O" routingKey body)) // TODO
                }
                let cts = new CancellationTokenSource()
                let resolveAndCancelTimeout result =
                    cts.Cancel()
                    resolve result

                Async.Start (timeout, cts.Token)
                this.Queries.TryAdd(correlationId, resolveAndCancelTimeout) |> ignore
            )

module RabbitSimpleMQ =

    let connect (moduleName:string) (cf: RabbitMQ.Client.ConnectionFactory) =
        let rmq = RabbitSimpleMQ (moduleName, cf)
        let mq = rmq :> SimpleMQ
        mq.QueryQueue(name = "", bindings = [
            "ping", fun body trace -> async {
                mq.PublishResponse(trace, moduleName, "text/plain")
            }

            moduleName, fun body trace -> async {
                return rmq.Queries
                |> DictUtil.tryRemove(trace.correlationId)
                |> Option.map (fun resolve -> resolve body)
                |> Option.defaultValue ()
            }
        ])
        mq
