module Main exposing (..)

import Html exposing (..)
import Html.App as App
import Html.Attributes exposing (..)


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
                            , button [ class "btn btn-default" ]
                                [ text "C:"
                                , text " "
                                , i [ class "fa fa-play" ] []
                                ]
                            , text " "
                            , button [ class "btn btn-info" ]
                                [ text "D:"
                                , text " "
                                , i [ class "fa fa-pause" ] []
                                ]
                            , text " "
                            , button [ class "btn btn-default" ]
                                [ text "E:"
                                , text " "
                                , i [ class "fa fa-refresh" ] []
                                ]
                            , text " "
                            , button [ class "btn btn-info" ]
                                [ text "F:"
                                , text " "
                                , i [ class "fa fa-step-forward" ] []
                                ]
                            ]
                        ]
                    ]
                ]
            ]
        , div [ class "container" ]
            [ div [ class "row" ]
                [ div [ class "panel panel-default" ]
                    [ div [ class "panel-heading" ]
                        [ h3 [ class "panel-title" ]
                            [ text "Results"
                            , text " "
                            , i [ class "fa fa-check" ] []
                            ]
                        ]
                    , div [ class "panel-body" ]
                        [ text "Panel content…" ]
                    ]
                , div [ class "panel panel-info" ]
                    [ div [ class "panel-heading" ]
                        [ h3 [ class "panel-title" ]
                            [ text "D:"
                            , text " "
                            , i [ class "fa fa-spinner fa-pulse" ] []
                            ]
                        ]
                    , div [ class "panel-body" ]
                        [ text "Panel content…" ]
                    ]
                ]
            ]
        ]
