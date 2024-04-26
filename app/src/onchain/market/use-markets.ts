import { useDataStore } from "@/contexts/anchor";
import { MarketData, Markets } from "./types";
import useSWR from "swr";
import { findMarketPDA } from "gmsol";
import { useMemo } from "react";
import { PublicKey } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";
import { isObject } from "lodash";

const MARKETS_KEY = "data_store/markets";

const BN_TWO = new BN(2);

export const useMarkets = (params?: { store: PublicKey, marketTokens: PublicKey[] }) => {
  const dataStore = useDataStore();

  const request = useMemo(() => {
    return params ? {
      key: MARKETS_KEY,
      marketAddresses: params.marketTokens.map(token => findMarketPDA(params.store, token)[0].toBase58()),
    } : null;
  }, [params]);

  const { data } = useSWR(request, async ({ marketAddresses }) => {
    const data = await dataStore.account.market.fetchMultiple(marketAddresses);
    const markets = (data ?? []).map(market => {
      if (market) {
        const isSingle = market.meta.longTokenMint.equals(market.meta.shortTokenMint);
        const data: MarketData = {
          marketTokenAddress: market.meta.marketTokenMint,
          indexTokenAddress: market.meta.indexTokenMint,
          longTokenAddress: market.meta.longTokenMint,
          shortTokenAddress: market.meta.shortTokenMint,
          longPoolAmount: isSingle ? market.pools.pools[0].longTokenAmount.div(BN_TWO) : market.pools.pools[0].longTokenAmount,
          shortPoolAmount: isSingle ? market.pools.pools[0].longTokenAmount.div(BN_TWO) : market.pools.pools[0].shortTokenAmount,
          isSingle,
        };
        return data;
      } else {
        return null;
      }
    });
    return markets.reduce((acc, market) => {
      if (market) {
        acc[market.marketTokenAddress.toBase58()] = market;
      }
      return acc;
    }, {} as Markets);
  });

  const markets = useMemo(() => {
    return data ?? {};
  }, [data]);

  return markets;
};

export const fitlerMarkets = (value: unknown) => {
  if (isObject(value)) {
    const { key } = value as { key?: string };
    if (key === MARKETS_KEY) {
      console.debug("filtered markets");
      return true;
    }
  }
  return false;
};
