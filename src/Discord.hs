{-# LANGUAGE OverloadedStrings #-}

module Discord (alert) where

import Network.HTTP.Simple
import Data.String

alert :: String -> IO ()
alert webhookUrl = do
  let request = setRequestMethod "POST"
       $ setRequestHeader "Content-Type" ["application/json"]
       $ setRequestBodyLBS "{\"content\":\"Alert!\"}"
       $ fromString webhookUrl

  response <- httpNoBody request
  return ()