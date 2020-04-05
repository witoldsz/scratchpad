namespace CurrencyOne.Infra

open RabbitMQ.Client
open RabbitMQ.Client.Events
open System
open System.Text
open System.Threading
open MQ3
open Utils

module internal RabbitMQ3 =

    type private QueueProps =
        { name: string
          exchangeName: string
          prefetchCount: int
          durable: bool
          exclusive: bool
          autoDelete: bool
          autoAck: bool }

        static member Query name: QueueProps =
            { name = name
              exchangeName = "amq.topic"
              prefetchCount = 10
              durable = false
              exclusive = true
              autoDelete = true
              autoAck = true }

        static member Event name prefetchCount: QueueProps =
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
            receivingChannel.BasicQos(prefetchSize = 0u, prefetchCount = uint16 p.prefetchCount, ``global`` = true)
            receivingChannel.QueueDeclare
                (queue = p.name, durable = p.durable, exclusive = p.exclusive, autoDelete = p.autoDelete)
            |> ignore

        interface IMQueue with

            member this.Bind(bindingKey, consumer) =
                logInfo "MQ: binding queue %s to %s" p.name bindingKey
                receivingChannel.QueueBind(p.name, p.exchangeName, bindingKey)
                bindings <- Binding(bindingKey, consumer) :: bindings
                this :> IMQueue

            member this.Done() =
                let eventConsumer = EventingBasicConsumer(receivingChannel)
                eventConsumer.Received.Add onEvent
                receivingChannel.BasicConsume(queue = p.name, autoAck = p.autoAck, consumer = eventConsumer) |> ignore
                logInfo "MQ: binding queue %s done" p.name

    type private RabbitMq3 (moduleName: string, cf: ConnectionFactory) as this =

        let conn = cf.CreateConnection()
        let publishChannel = conn.CreateModel()
        let queries = Collections.Concurrent.ConcurrentDictionary<Guid, Body -> unit>()

        let queue (p: QueueProps) =
            logInfo "MQ: creating queue %s" p.name
            RabbitMQueue(conn, p) :> IMQueue

        let createPublishProps (trace: Trace) =
            let props = publishChannel.CreateBasicProperties()
            props.Headers <- Map.ofList [
                "trace", trace.TracePoints :> obj
                "publisher", moduleName :> obj
                "ts", DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ss.fffZ") :> obj
            ]
            props.ContentType <- "application/json"
            props

        let publish exchange routingKey props (body: string) =
            fun () -> publishChannel.BasicPublish(exchange, routingKey, props, Encoding.UTF8.GetBytes body)
            |> lock publishChannel

        do
            (this :> IMQ3).QueryQueue(moduleName)
                .Bind("ping", fun body trace -> failwith "TODO…")
                .Bind(moduleName, fun body trace -> async {
                    return queries
                    |> DictUtil.tryRemove(trace.CorrelationId)
                    |> Option.map (fun resolve -> resolve body)
                    |> Option.defaultValue ()
                })
                .Done()

        interface IMQ3 with

            member this.QueryQueue name =
                queue (QueueProps.Query name)

            member this.EventQueue name prefetchCount =
                queue (QueueProps.Event name prefetchCount)

            member this.Publish routingKey body oldTrace =
                let trace = oldTrace.Next()
                let props = createPublishProps trace
                props.Persistent <- true
                publish "amq.topic" routingKey props body
                trace

            member this.PublishResponse oldTrace body =
                let trace = oldTrace.Next()
                let props = createPublishProps trace
                props.Persistent <- false
                props.CorrelationId <- trace.CorrelationId.ToString()
                publish "" trace.ReplyTo props body

            member this.Query routingKey body oldTrace =
                let trace = oldTrace.Next()
                let correlationId = Guid.NewGuid()
                let props = createPublishProps trace
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

    let connect (moduleName:string) (cf: RabbitMQ.Client.ConnectionFactory) =
        RabbitMq3 (moduleName, cf) :> IMQ3