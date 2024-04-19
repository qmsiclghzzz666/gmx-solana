import { PublicKey } from "@solana/web3.js";

import { Market as ParsedMarket } from "states/market";

export interface GMSOLDeployment {
  store: string,
  oracle: string,
  markets: Market[],
}

export interface Market {
  name: string,
  market_token: string,
  index_token: string,
  long_token: string,
  short_token: string,
}

export interface ParsedGMSOLDeployment {
  store: PublicKey,
  oracle: PublicKey,
  markets: ParsedMarket[],
}

export const getGMSOLDeployment = () => {
  const deployment = window.__GMSOL_DEPLOYMENT__;

  if (deployment) {
    const parsed: ParsedGMSOLDeployment = {
      store: new PublicKey(deployment.store),
      oracle: new PublicKey(deployment.oracle),
      markets: deployment.markets.map(parseMarket),
    };
    return parsed;
  }
}

const parseMarket = (market: Market) => {
  return {
    marketTokenAddress: new PublicKey(market.market_token),
    indexTokenAddress: new PublicKey(market.index_token),
    longTokenAddress: new PublicKey(market.long_token),
    shortTokenAddress: new PublicKey(market.short_token),
  } as ParsedMarket;
}
