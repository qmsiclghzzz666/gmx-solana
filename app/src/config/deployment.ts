import { PublicKey } from "@solana/web3.js";

export interface GMSOLDeployment {
  store: PublicKey,
  oracle: PublicKey,
  markets: Market[],
}

export interface Market {
  name: string,
  market_token: PublicKey,
  index_token: PublicKey,
  long_token: PublicKey,
  short_token: PublicKey,
}
