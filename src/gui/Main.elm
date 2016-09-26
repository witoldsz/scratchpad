import Html exposing (..)
import Html.App as App
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import WebSocket

main =
  App.program
    { init = init
    , view = view
    , update = update
    , subscriptions = subscriptions
    }

wsServerUrl =
  "ws://localhost:3000/ws"

-- MODEL
type alias Model =
  { input : String
  , messages : List String
  , count : Int
  }

init : (Model, Cmd Msg)
init =
  (Model "" [] 0, Cmd.none)

-- UPDATE
type Msg
  = Input String
  | Send
  | NewMessage String


update : Msg -> Model -> (Model, Cmd Msg)
update msg {input, messages, count} =
  case msg of
    Input newInput ->
      (Model newInput messages 0, Cmd.none)

    Send ->
      (Model "" messages 0, WebSocket.send wsServerUrl input)

    NewMessage str -> 
      (Model input (str :: messages) (count + 1), Cmd.none)

-- SUBSCRIPTIONS
subscriptions : Model -> Sub Msg
subscriptions model =
  WebSocket.listen wsServerUrl NewMessage

-- VIEW
view : Model -> Html Msg
view model =
  div []
    [ input [onInput Input, value model.input] []
    , button [onClick Send] [text "Send"]
    , text (toString model.count)
    , div [] (List.map viewMessage (List.reverse model.messages))
    ]


viewMessage : String -> Html msg
viewMessage msg =
  div [] [ text msg ]
