namespace CurrencyOne.Infra

open RabbitMQ.Client
open RabbitMQ.Client.Events
open System
open System.Text
open System.Threading
open MQ3
open Utils

module RabbitMQ3 =

    type private QueueProps =
        { name: string
          exchangeName: string
          prefetchCount: int
          durable: bool
          exclusive: bool
          autoDelete: bool
          autoAck: bool }

        static member Query(name): QueueProps =
            { name = name
              exchangeName = "amq.topic"
              prefetchCount = 10
              durable = false
              exclusive = true
              autoDelete = true
              autoAck = true }

        static member Event(name, prefetchCount): QueueProps =
            { name = name
              exchangeName = "amq.topic"
              prefetchCount = prefetchCount
              durable = true
              exclusive = false
              autoDelete = false
              autoAck = false }

    type private RabbitMQueue (ch: IModel, p: QueueProps) =

        do ch.BasicQos(prefetchSize = 0u, prefetchCount = uint16 p.prefetchCount, ``global`` = true)
        do ch.QueueDeclare(queue = p.name, durable = p.durable, exclusive = p.exclusive, autoDelete = p.autoDelete)
            |> ignore

        member val bindings: Binding list = List.empty with get, set
        member val consumer = EventingBasicConsumer(ch)

        member this.onEvent(event: BasicDeliverEventArgs) =
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
                do! this.bindings
                    |> List.tryFind (fun b -> b.Matches routingKey)
                    |> Option.map (fun b -> b.Consumer body trace)
                    |> Option.defaultValue (async { return () })

                if p.autoAck
                then ch.BasicAck(deliveryTag = event.DeliveryTag, multiple = false)
            })

        interface IMQueue with

            member this.Bind(bindingKey, consumer) =
                logInfo "MQ: binding queue %s to %s" p.name bindingKey
                ch.QueueBind(p.name, p.exchangeName, bindingKey)
                this.bindings <- Binding(bindingKey, consumer) :: this.bindings
                this :> IMQueue

            member this.Done() =
                logInfo "MQ: binding queue %s done" p.name
                this.consumer.Received.Add(this.onEvent)
                ch.BasicConsume(queue = p.name, autoAck = p.autoAck, consumer = this.consumer) |> ignore

    type private RabbitMq3 (moduleName: string, conn: IConnection, publishChannel: IModel) as this =

        let mq3 = this :> IMQ3

        member val queries: Map<Guid, Body -> unit> = Map.empty with get, set

        member this.Queue(p: QueueProps) =
            logInfo "MQ: creating queue %s" p.name
            RabbitMQueue(conn.CreateModel(), p)

        member this.CreatePublishProps(trace: Trace) =
            let props = publishChannel.CreateBasicProperties()
            props.Headers <- Map.ofList [
                "trace", trace.TracePoints :> obj
                "publisher", moduleName :> obj
                "ts", DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ss.fffZ") :> obj
            ]
            props.ContentType <- "application/json"
            props

        interface IMQ3 with

            member this.QueryQueue name =
                this.Queue(QueueProps.Query(name)) :> IMQueue

            member this.EventQueue name prefetchCount =
                this.Queue(QueueProps.Event(name, prefetchCount)) :> IMQueue

            member this.Publish routingKey body oldTrace =
                let trace = oldTrace.Next()
                let props = this.CreatePublishProps(trace)
                props.Persistent <- true
                publishChannel.BasicPublish("amq.topic", routingKey, props, Encoding.UTF8.GetBytes body)
                trace

            member this.PublishResponse oldTrace body =
                let trace = oldTrace.Next()
                let props = this.CreatePublishProps(trace)
                props.Persistent <- false
                props.CorrelationId <- trace.CorrelationId.ToString()
                publishChannel.BasicPublish("amq.topic", trace.ReplyTo, props, Encoding.UTF8.GetBytes body)

            member this.Query routingKey body oldTrace =
                let trace = oldTrace.Next()
                let correlationId = Guid.NewGuid()
                let props = this.CreatePublishProps(trace)
                props.ReplyTo <- moduleName
                props.CorrelationId <- correlationId.ToString()
                props.Persistent <- false
                publishChannel.BasicPublish("amq.topic", routingKey, props, Encoding.UTF8.GetBytes body)
                Async.FromContinuations (fun (resolve, reject, _) ->
                    let timeout = async {
                        do! Async.Sleep 3000
                        this.queries <- this.queries.Remove(correlationId)
                        reject (System.TimeoutException (sprintf "routing key: %s query body: %O" routingKey body)) // TODO
                    }
                    let cts = new CancellationTokenSource()
                    let resolveAndCancelTimeout result =
                        cts.Cancel()
                        this.queries <- this.queries.Remove(correlationId)
                        resolve result

                    Async.Start (timeout, cts.Token)
                    this.queries <- this.queries.Add(correlationId, resolveAndCancelTimeout)
                )

    let connect (moduleName:string) (cf: ConnectionFactory) =
        let conn = cf.CreateConnection()
        let publishChannel = conn.CreateModel()
        let rmq3 = RabbitMq3(moduleName, conn, publishChannel)
        let mq3 = rmq3 :> IMQ3
        mq3.QueryQueue(moduleName)
            .Bind("ping", fun body trace -> failwith "TODO…")
            .Bind(moduleName, fun body trace -> async {
                return trace.CorrelationId
                |> rmq3.queries.TryFind
                |> Option.map (fun resolve -> resolve body)
                |> Option.defaultValue ()
            })
            .Done()
        mq3