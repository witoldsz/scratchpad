{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE DeriveGeneric     #-}
{-# LANGUAGE FlexibleInstances #-}

module Semux (getLastCoinbase) where

import Data.Text
import Data.List
import Data.Time
import Data.Either
import Data.Maybe
import Data.Int
import Control.Applicative
import Data.Aeson
import Data.Aeson.Types
import GHC.Generics
import GHC.List
import qualified Data.ByteString.Lazy as B
import Network.HTTP.Conduit (simpleHttp)

apiUrl = "https://semux.online/v2.1.0/"
pageSize = 100

getLastCoinbase :: String -> IO UTCTime
getLastCoinbase delegate = do
  account <- getRight $ getAccount delegate
  let txCount = (transactionCount . result) account
  tx <- findLastCoinbase txCount
  print tx
  return $ timestamp tx

  where
  findLastCoinbase :: Int -> IO Transaction
  findLastCoinbase lastTx = do
    response <- getRight $ getTransactions delegate txRange
    let txsRev = (Data.List.reverse . result) response
    let lastCoinbase = Data.List.find (\i -> transactionType i == "COINBASE") txsRev
    solution lastCoinbase
    where
      from = max 0 (lastTx - pageSize)
      txRange = (from, lastTx)

      solution :: Maybe Transaction -> IO Transaction
      solution (Just coinbase ) =
        return coinbase
      solution Nothing =
        if (from > 0)
          then findLastCoinbase from
          else fail "No COINBASE found"

getRight :: IO (Either String a) -> IO a
getRight x =
  x >>= either fail return

-- Account
data Account = Account
  { transactionCount :: Int
  } deriving (Show, Generic)

instance FromJSON Account

getAccount :: String -> IO (Either String (Response Account))
getAccount addr =
  eitherDecode <$> fetchJson
  where
    fetchJson :: IO B.ByteString
    fetchJson =
      putStrLn url >> simpleHttp url
      where
        url = apiUrl ++ "account?address=" ++ addr

-- Transaction
data Transaction = Transaction
 { timestamp :: UTCTime
 , transactionType :: !Text
 } deriving (Show)

instance FromJSON Transaction where
  parseJSON (Object v) = Transaction
    <$> fmap textToUTC (v .: "timestamp")
    <*> v .: "type"

getTransactions :: String -> (Int, Int) -> IO (Either String (Response [Transaction]))
getTransactions addr (from, to) =
  eitherDecode <$> fetchJson
  where
    fetchJson :: IO B.ByteString
    fetchJson =
      putStrLn url >> simpleHttp url
      where
        url =
          apiUrl ++ "account/transactions?address=" ++ addr ++ "&from=" ++ (show from) ++ "&to=" ++ (show to)

-- Response
data Response a = Response
  { success :: Bool
  , message :: !Text
  , result :: a
  } deriving (Show, Generic)

instance FromJSON (Response [Transaction])
instance FromJSON (Response Account)

-- UTCTime
textToUTC :: Text -> UTCTime
textToUTC = fromJust . (parseTimeM False defaultTimeLocale "%s") . unpack . (Data.Text.take 10)
