module Scratchpad

open RabbitMQ.Client
open RabbitMQ.Client.Events

type Mq3 = unit

type Queue = unit

type EventHandler = unit

type MqConnect = ConnectionFactory -> Mq3

type QueueProps =
    { name: string
      exchangeName: string
      prefetchCount: int
      durable: bool
      exclusive: bool
      autoDelete: bool
      autoAck: bool }

let queue (conn: IConnection) (props: QueueProps) (mq3: Mq3): Mq3 =
    let ch = conn.CreateModel()
    ch.BasicQos(prefetchSize = 0u, prefetchCount = uint16 props.prefetchCount, ``global`` = true)
    ch.QueueDeclare
        (queue = props.name, durable = props.durable, exclusive = props.exclusive, autoDelete = props.autoDelete)
    |> ignore


let connect (cf: ConnectionFactory): Mq3 =
    let conn = cf.CreateConnection()
    let publishChannel = conn.CreateModel()

    failwith "create mq3"

let eventQueue (queueName: string) (mq3: Mq3): Queue =
    failwith "eventQueue"

let bind (routingKey: string) (queue: Queue) (h: EventHandler): unit =
    failwith "bind"

let event_order_MARKET_REQUESTED = ()

let main argv =
    let cf = ConnectionFactory(HostName = "localhost")
    let mq3 = connect cf
    mq3
    |> eventQueue "orders"
    |> bind "event.orders.MARKET_REQUESTED" event_order_MARKET_REQUESTED
    ()
