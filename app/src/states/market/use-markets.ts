import { GMSOL_DEPLOYMENT } from "@/config/deployment";
import { useDataStore } from "@/contexts/anchor";
import { MarketData, Markets } from "./types";
import useSWR from "swr";
import { findMarketPDA } from "gmsol";
import { useMemo } from "react";

export const useMarkets = () => {
  const dataStore = useDataStore();

  const marketAddresses = useMemo(() => {
    const deployment = GMSOL_DEPLOYMENT;
    return deployment ? {
      key: "data_store/markets",
      marketAddresses: deployment.marketTokens.map(token => findMarketPDA(deployment.store, token)[0]),
    } : null;
  }, []);

  const { data } = useSWR(marketAddresses, async ({ marketAddresses }) => {
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
