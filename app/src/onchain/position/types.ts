import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { MarketInfo } from "../market";

export interface Position {
  address: PublicKey,
  owner: PublicKey,
  marketTokenAddress: PublicKey,
  collateralTokenAddress: PublicKey,
  isLong: boolean,
  sizeInUsd: BN,
  sizeInTokens: BN,
  isOpening?: boolean,
}

export interface Positions {
  [address: string]: Position,
}

export type PositionInfo = Position & {
  marketInfo: MarketInfo,
};

export interface PositionInfos {
  [address: string]: PositionInfo,
}
