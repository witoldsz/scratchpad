module Main exposing (..)

import Html exposing (text)
import Dict exposing (Dict)


main =
    text "Hello World"



-- MODEL


type alias Model =
    { input : String
    , messages : List String
    , count : Int
    , db : Db
    }


type alias Item =
    { name : String
    , path : String
    , items : List Item
    }


type alias Db =
    Dict Int Dir


type alias File =
    { name : String }


type Dir
    = Dir Int String (List Dir) (List File)



-- UPDATE
