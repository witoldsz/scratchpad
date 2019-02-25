{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE DeriveGeneric     #-}
{-# LANGUAGE FlexibleInstances #-}

module Main where

import Data.Text
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

apiUrl =
  "https://semux.online/v2.1.0/"

delegate =
  "0xdb7cadb25fdcdd546fb0268524107582c3f8999c"

main :: IO ()
main = do
  txCount <- getTxCount delegate
  txs <- getRight $ getTransactions delegate (max 0 (txCount - 10), txCount)
  print (GHC.List.last (result txs))
  return ()

getTxCount :: String -> IO Int
getTxCount addr =
  parseTxCount <$> fetchJson
  where
    fetchJson :: IO B.ByteString
    fetchJson =
      simpleHttp (apiUrl ++ "account?address=" ++ addr)

    parseTxCount :: B.ByteString -> Int
    parseTxCount json =
      fromRight 0 (eitherDecode json >>= parseEither parser)
      where
        parser :: Value -> Parser Int
        parser (Object o) =
          (o .: "result") >>= (.: "transactionCount")

getTransactions :: String -> (Int, Int) -> IO (Either String (Response [Transaction]))
getTransactions addr (from, to) =
  eitherDecode <$> fetchJson
  where
    fetchJson :: IO B.ByteString
    fetchJson =
      simpleHttp (apiUrl ++ "account/transactions?address=" ++ addr ++ "&from=" ++ (show from) ++ "&to=" ++ (show to))

getRight :: IO (Either String a) -> IO a
getRight x =
  x >>= either fail return

data Response a = Response
  { success :: Bool
  , message :: !Text
  , result :: a
  } deriving (Show, Generic)

instance FromJSON (Response [Transaction])

data Transaction = Transaction
 { timestamp :: UTCTime
 , typ :: !Text
 } deriving (Show)

instance FromJSON Transaction where
  parseJSON (Object v) = Transaction
    <$> fmap parseTs (v .: "timestamp")
    <*> v .: "type"

parseTs :: Text -> UTCTime
parseTs = fromJust . (parseTimeM False defaultTimeLocale "%s") . unpack . (Data.Text.take 10)