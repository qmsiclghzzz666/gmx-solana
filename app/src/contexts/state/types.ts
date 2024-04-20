import { BN } from "@coral-xyz/anchor";
import { MarketInfo } from "@/states/market";
import { TokenData } from "@/states/token";

export interface MarketStat {
  marketInfo: MarketInfo;
  poolValueUsd: BN;
  usedLiquidity: BN;
  maxLiquidity: BN;
  netFeeLong: BN;
  netFeeShort: BN;
  utilization: BN;
}

export interface IndexTokenStat {
  token: TokenData;
  price: BN;
  totalPoolValue: BN;
  totalUtilization: BN;
  totalUsedLiquidity: BN;
  totalMaxLiquidity: BN;
  bestNetFeeLong: BN;
  bestNetFeeShort: BN;
  /**
   * Sorted by poolValueUsd descending
   */
  marketsStats: MarketStat[];
}
