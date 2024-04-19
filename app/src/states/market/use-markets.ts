import { getGMSOLDeployment } from "config/deployment";
import { useDataStore } from "contexts/anchor";
import { MarketData, Markets } from "./types";
import useSWR from "swr";
import { findMarketPDA } from "gmsol";

export const useMarkets = () => {
  const deployment = getGMSOLDeployment();
  const dataStore = useDataStore();

  const { data } = useSWR("data_store/markets", async () => {
    if (!deployment) {
      return [];
    }
    const { store, markets } = deployment;
    return await dataStore.account.market.fetchMultiple(markets.map(market => findMarketPDA(store, market.marketTokenAddress)[0]));
  });
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
};
