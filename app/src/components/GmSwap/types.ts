import { MarketInfo } from "@/onchain/market";
import { TokenData, Tokens } from "@/onchain/token";
import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

export enum Operation {
  Deposit = "Deposit",
  Withdrawal = "Withdrawal",
}

export enum Mode {
  Single = "Single",
  Pair = "Pair",
}

export interface CreateDepositParams {
  marketToken: PublicKey,
  initialLongToken: PublicKey,
  initialShortToken: PublicKey,
  initialLongTokenAmount: BN,
  initialShortTokenAmount: BN,
}

export interface CreateWithdrawalParams {
  marketToken: PublicKey,
  amount: BN,
  finalLongToken: PublicKey,
  finalShortToken: PublicKey,
}

export interface GmState {
  market: MarketInfo,
  operation: Operation,
  mode: Mode,
  firstToken?: TokenData,
  secondToken?: TokenData,
  marketToken?: TokenData,
  marketTokens?: Tokens,
  sortedMarketsInfoByIndexToken: MarketInfo[],
  input: InputState,
}

export interface InputState {
  firstTokenInputValue: string,
  secondTokenInputValue: string,
  marketTokenInputValue: string,
}

export interface Action {
  type:
  "reset"
  | "set-first-token-input-value"
  | "set-second-token-input-value"
  | "set-market-token-input-value",
  value?: string,
}
