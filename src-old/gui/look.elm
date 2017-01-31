module Main exposing (..)

import Html exposing (..)
import Html.App as App
import Html.Attributes exposing (..)


btn : String -> String -> String -> Html a
btn btnClass btnText faIcon =
    button [ class ("btn btn-" ++ btnClass) ]
        [ text btnText
        , text " "
        , i [ class ("fa fa-" ++ faIcon) ] []
        ]


panel : String -> List (Html a) -> List (Html a) -> Html a
panel pClass pTitle pBody =
    div [ class ("panel panel-" ++ pClass) ]
        [ div [ class "panel-heading" ] [ h3 [ class "panel-title" ] pTitle ]
        , div [ class "panel-body" ] pBody
        ]


main =
    div []
        [ nav [ class "navbar navbar-inverse navbar-fixed-top" ]
            [ div [ class "container" ]
                [ div [ class "navbar-header" ]
                    [ a [ class "navbar-brand", href "#" ]
                        [ text ("Games Mover") ]
                    ]
                ]
            ]
        , div [ class "container" ]
            [ div [ class "row" ]
                [ div [ class "panel panel-default" ]
                    [ div [ class "panel-body" ]
                        [ div [ class "form-inline" ]
                            [ label [ class "control-label" ] [ text "Scan" ]
                            , text " "
                            , btn "default" "C:" "play"
                            , text " "
                            , btn "info" "D:" "pause"
                            , text " "
                            , btn "default" "E:" "refresh"
                            , text " "
                            , btn "info" "F:" "step-forward"
                            ]
                        ]
                    ]
                ]
            ]
        , div [ class "container" ]
            [ div [ class "row" ]
                [ panel "default"
                    [ text "Games" ]
                    [ text "Panel content…" ]
                , panel "info"
                    [ text "D:"
                    , text " "
                    , i [ class "fa fa-spinner fa-pulse" ] []
                    ]
                    [ div [ class "jstree-default" ]
                        [ ul [ class "jstree-container-ul jstree-children" ]
                            [ leaf "1"
                            , leaf "2"
                            , dir "3" [leaf "31"]
                            , li [] [ text "4", ul [] [ li [] [ text "11" ] ] ]
                            ]
                        ]
                    ]
                ]
            ]
        ]

icon cls =
    i [class cls] []

dir name childs =
    li [ class "jstree-node" ] [ text name, ul [] childs ]


leaf name =
    li [ class "jstree-node" ]
        [ icon "jstree-icon"
        , a [ class "jstree-anchor" ] 
            [ icon "jstree-icon jstree-themeicon"
            , text name 
            ] 
        ]
