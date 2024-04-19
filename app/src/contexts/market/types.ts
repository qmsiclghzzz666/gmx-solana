import { PublicKey } from "@solana/web3.js";
import { TokenData } from "contexts/token";

export interface Market {
  marketTokenAddress: PublicKey,
  indexTokenAddress: PublicKey,
  longTokenAddress: PublicKey,
  shortTokenAddress: PublicKey,
}

export interface MarketState {
  longToken: TokenData,
  shortToken: TokenData,
  indexToken: TokenData,
}

export type MarketInfo = Market & MarketState;
