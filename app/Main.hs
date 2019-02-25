{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE FlexibleInstances #-}

module Main where

import Semux
import System.Environment
import Data.Time.Clock
import Control.Monad

main :: IO ()
main = do
  delegate <- getEnv "DELEGATE"

  lastCoinbase <- getLastCoinbase delegate
  now <- getCurrentTime
  let diff = diffSeconds now lastCoinbase
  putStrLn $ "Last COINBASE was " ++ show lastCoinbase ++ " that is " ++ show diff ++ " seconds ago"

  when (diff > 3600) alert

  where
    alert =
      undefined -- TODO

diffSeconds t1 t2 =
  let (res, _) = properFraction $ diffUTCTime t1 t2
  in res
