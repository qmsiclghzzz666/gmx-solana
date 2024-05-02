import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { MarketInfo } from "../market";
import { TokenData } from "../token";

export interface Position {
  address: PublicKey,
  owner: PublicKey,
  marketTokenAddress: PublicKey,
  collateralTokenAddress: PublicKey,
  isLong: boolean,
  sizeInUsd: BN,
  sizeInTokens: BN,
  collateralAmount: BN,
  isOpening?: boolean,
}

export interface Positions {
  [address: string]: Position,
}

export type PositionInfo = Position & {
  marketInfo: MarketInfo,
  collateralToken: TokenData,
  entryPrice?: BN,
  markPrice?: BN,
  remainingCollateralUsd?: BN,
  remainingCollateralAmount?: BN,
  netValue?: BN,
  leverage?: BN,
  pnl?: BN,
  pnlPercentage?: BN,
  uiFeeUsd?: BN,
  liquidationPrice?: BN,
};

export interface PositionInfos {
  [address: string]: PositionInfo,
}
