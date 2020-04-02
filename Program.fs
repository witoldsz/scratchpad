module Main

open System
open RabbitMQ.Client
open RabbitMQ.Client.Events
open System.Text

let connect (cf: ConnectionFactory) =
    let connection = cf.CreateConnection()
    let publishChannel = connection.CreateModel()

    0

let publishQuery publishChannel queries routingKey =

    0

[<EntryPoint>]
let main argv =
    printfn "Hello World from F#!"
    Console.WriteLine(System.Threading.Thread.CurrentThread.Name)

    let factory = ConnectionFactory(HostName = "localhost")
    use connection = factory.CreateConnection()
    use channel = connection.CreateModel()
    channel.BasicQos(prefetchSize = 0u, prefetchCount = 1us, ``global`` = true)

    let consumer = EventingBasicConsumer(channel)

    consumer.Received.Add(fun event ->
        printf "Consumer 1 @ %s" System.Threading.Thread.CurrentThread.Name
        let headers = event.BasicProperties.Headers
        let body = event.Body
        //   printf "%A" headers
        System.Threading.Thread.Sleep 1000
        Console.WriteLine(Encoding.UTF8.GetString(body))
        channel.BasicAck(deliveryTag = event.DeliveryTag, multiple = false))

    // consumer.Received.Add(fun event ->
    //     printfn "Consumer 2"
    //     Console.WriteLine(System.Threading.Thread.CurrentThread.Name)
    //     let headers = event.BasicProperties.Headers
    //     let body = event.Body
    //     //   printf "%A" headers
    //     System.Threading.Thread.Sleep 1000
    //     Console.WriteLine(Encoding.UTF8.GetString(body))
    //     channel.BasicAck(deliveryTag = event.DeliveryTag, multiple = false))

    channel.QueueDeclare(queue = "hello", durable = false, exclusive = false, autoDelete = false) |> ignore

    channel.BasicConsume(queue = "hello", autoAck = false, consumer = consumer) |> ignore

    System.Threading.Thread.Sleep 100000

    // while true do
    //     let ea = consumer.Queue.Dequeue() :> BasicDeliverEventArgs
    //     let body = ea.Body
    //     let message = Encoding.UTF8.GetString(body)
    //     printfn "%s" message

    0 // return an integer exit code
