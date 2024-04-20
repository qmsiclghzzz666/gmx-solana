import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { TokenData } from "@/onchain/token";

export interface Market {
  marketTokenAddress: PublicKey,
  indexTokenAddress: PublicKey,
  longTokenAddress: PublicKey,
  shortTokenAddress: PublicKey,
}

export interface MarketTokens {
  indexToken: TokenData,
  longToken: TokenData,
  shortToken: TokenData,
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

export interface MarketInfos {
  [marketTokenAddress: string]: MarketInfo;
}
