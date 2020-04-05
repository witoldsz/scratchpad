namespace SimpleMQ

module internal Utils =
    let logInfo =
        printfn

    module DictUtil =

        open System.Collections

        let tryRemove key (dict: Concurrent.ConcurrentDictionary<'a, 'b>) =
            match dict.TryRemove(key) with
            | (true, v) -> Some v
            | (false, _) -> None