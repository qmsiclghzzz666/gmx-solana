import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { TokenData } from "@/states/token";

export interface Market {
  marketTokenAddress: PublicKey,
  indexTokenAddress: PublicKey,
  longTokenAddress: PublicKey,
  shortTokenAddress: PublicKey,
}

export interface MarketTokens {
  longToken: TokenData,
  shortToken: TokenData,
  indexToken: TokenData,
}

export interface MarketState {
  longPoolAmount: BN,
  shortPoolAmount: BN,
}

export type MarketData = Market & MarketState;

export type MarketInfo = MarketData & MarketTokens;

export interface Markets {
  [marketTokenAddress: string]: MarketData;
}
