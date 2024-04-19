import { PublicKey } from "@solana/web3.js";

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

export interface ParsedMarket {
  name: string,
  market_token: PublicKey,
  index_token: PublicKey,
  long_token: PublicKey,
  short_token: PublicKey,
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
    market_token: new PublicKey(market.market_token),
    index_token: new PublicKey(market.index_token),
    long_token: new PublicKey(market.long_token),
    short_token: new PublicKey(market.short_token),
  } as ParsedMarket;
}
