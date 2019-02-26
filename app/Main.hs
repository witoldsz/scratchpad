{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE FlexibleInstances #-}

module Main where

import Semux (getLastCoinbase)
import Discord (alert)
import System.Environment
import Data.Time.Clock
import Control.Monad

main :: IO ()
main = do
  semuxApi <- getEnv "SEMUX_API"
  delegate <- getEnv "DELEGATE"
  webhookUrl <- getEnv "WEBHOOK_URL"

  lastCoinbase <- getLastCoinbase semuxApi delegate
  now <- getCurrentTime
  let diff = diffSeconds now lastCoinbase
  putStrLn $ "Last COINBASE was " ++ show lastCoinbase ++ " that is " ++ show diff ++ " seconds ago"

  when (diff > 3600) (alert webhookUrl)

diffSeconds t1 t2 =
  let (res, _) = properFraction $ diffUTCTime t1 t2
  in res
