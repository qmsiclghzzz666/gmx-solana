import { useDataStore } from "@/contexts/anchor";
import { MarketData, Markets } from "./types";
import useSWR from "swr";
import { findMarketPDA } from "gmsol";
import { useMemo } from "react";
import { PublicKey } from "@solana/web3.js";

export const useMarkets = (params?: { store: PublicKey, marketTokens: PublicKey[] }) => {
  const dataStore = useDataStore();

  const request = useMemo(() => {
    return params ? {
      key: "data_store/markets",
      marketAddresses: params.marketTokens.map(token => findMarketPDA(params.store, token)[0]),
    } : null;
  }, [params]);

  const { data } = useSWR(request, async ({ marketAddresses }) => {
    const data = await dataStore.account.market.fetchMultiple(marketAddresses);
    const markets = (data ?? []).map(market => {
      return market ? {
        marketTokenAddress: market.meta.marketTokenMint,
        indexTokenAddress: market.meta.indexTokenMint,
        longTokenAddress: market.meta.longTokenMint,
        shortTokenAddress: market.meta.shortTokenMint,
        longPoolAmount: market.pools.pools[0].longTokenAmount,
        shortPoolAmount: market.pools.pools[0].shortTokenAmount,
      } as MarketData : null;
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
