import { BN } from "@coral-xyz/anchor";
import { MarketInfo, MarketInfos } from "@/onchain/market";
import { TokenData, Tokens } from "@/onchain/token";
import { TradeBoxState } from "@/onchain/trade";

export interface SharedStates {
  chainId?: string,
  market: MarketState,
  tradeBox: TradeBoxState,
}

export interface MarketState {
  marketInfos: MarketInfos,
  tokens: Tokens,
  marketTokens: Tokens,
}

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
