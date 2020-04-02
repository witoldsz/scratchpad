namespace CurrencyOne.Mq3

open RabbitMQ.Client
open System
open System.Threading

type QueueProps =
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

type RabbitMQueue () =
    interface IMQueue with
        member this.Bind(routingKey, consumer) = failwith "TODO"
        member this.Done() = ()

type RabbitMq3 private (moduleName: string, conn: IConnection, publishChannel: IModel) as this =

    let mq3 = this :> IMq3
    member val queries: Map<Guid, Body -> unit> = Map.empty with get, set

    static member Connect(moduleName:string, cf: ConnectionFactory) =
        let conn = cf.CreateConnection()
        let publishChannel = conn.CreateModel()
        let this = RabbitMq3(moduleName, conn, publishChannel)
        let mq3 = this :> IMq3
        mq3.QueryQueue(moduleName)
            .Bind("ping", fun body trace -> failwith "TODO…")
            .Bind(moduleName, fun body trace -> async {
                return trace.CorrelationId
                |> this.queries.TryFind
                |> Option.map (fun resolve -> resolve body)
                |> Option.defaultValue ()
            })
            .Done()
        mq3

    member this.Queue(p: QueueProps) =
        let ch = conn.CreateModel()
        ch.BasicQos(prefetchSize = 0u, prefetchCount = uint16 p.prefetchCount, ``global`` = true)
        ch.QueueDeclare(queue = p.name, durable = p.durable, exclusive = p.exclusive, autoDelete = p.autoDelete)
        |> ignore
        RabbitMQueue()

    interface IMq3 with

        member this.QueryQueue name =
            this.Queue(QueueProps.Query(name)) :> IMQueue

        member this.EventQueue name prefetchCount =
            this.Queue(QueueProps.Event(name, prefetchCount)) :> IMQueue

        member this.Publish routingKey body oldTrace =
            let trace = oldTrace.Next()
            let props = publishChannel.CreateBasicProperties()
            props.Headers <- Map.ofList [
                "trace", trace.TracePoints :> obj
                "publisher", moduleName :> obj
                "ts", DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ss.fffZ") :> obj
            ]
            props.ContentType <- "application/json"
            props.Persistent <- true
            publishChannel.BasicPublish("amq.topic", routingKey, props, body)
            trace

        member this.PublishResponse oldTrace body =
            let trace = oldTrace.Next()
            let props = publishChannel.CreateBasicProperties()
            props.Headers <- Map.ofList [
                "trace", trace.TracePoints :> obj
                "publisher", moduleName :> obj
                "ts", DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ss.fffZ") :> obj
            ]
            props.ContentType <- "application/json"
            props.Persistent <- false
            props.CorrelationId <- trace.CorrelationId.ToString()
            publishChannel.BasicPublish("amq.topic", trace.ReplyTo, props, body)

        member this.Query routingKey body oldTrace =
            let trace = oldTrace.Next()
            let correlationId = Guid.NewGuid()
            let props = publishChannel.CreateBasicProperties()
            props.Headers <- Map.ofList [
                "trace", trace.TracePoints :> obj
                "publisher", moduleName :> obj
                "ts", DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ss.fffZ") :> obj
            ]
            props.ReplyTo <- moduleName
            props.ContentType <- "application/json"
            props.CorrelationId <- correlationId.ToString()
            props.Persistent <- false
            publishChannel.BasicPublish("amq.topic", routingKey, props, body)
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
