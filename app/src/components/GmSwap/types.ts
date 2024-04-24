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
