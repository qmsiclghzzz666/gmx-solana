import { Address, BN } from "@coral-xyz/anchor";
import { MarketInfo } from "../market";
import { Token, TokenData, Tokens } from "../token";
import { useTradeBoxState } from "./use-trade-box-state"

export enum TradeType {
  Long = "Long",
  Short = "Short",
  Swap = "Swap",
}

export enum TradeMode {
  Market = "Market",
  Limit = "Limit",
  Trigger = "Trigger",
}

export interface TradeOptions {
  tradeType: TradeType,
  tradeMode: TradeMode,
  tokens: {
    indexTokenAddress?: string,
    fromTokenAddress?: string,
    swapToTokenAddress?: string,
  }
  markets: {
    [marketTokenAddress: string]: {
      longTokenAddress: string,
      shortTokenAddress: string,
    }
  },
  collateralAddress?: string,
}

export interface TradeParams {
  tradeType?: TradeType,
  tradeMode?: TradeMode,
  fromTokenAddress?: string;
  toTokenAddress?: string;
  marketAddress?: string;
  collateralAddress?: string;
}

export type TradeBoxState = ReturnType<typeof useTradeBoxState>;

export interface AvailableTokenOptions {
  tokens: Tokens,
  swapTokens: TokenData[],
  indexTokens: TokenData[],
  sortedIndexTokensWithPoolValue: string[],
  sortedLongAndShortTokens: string[],
  sortedAllMarkets: MarketInfo[],
}

export interface TradeFlags {
  isLong: boolean;
  isShort: boolean;
  isSwap: boolean;
  /**
   * ```ts
   * isLong || isShort
   * ```
   */
  isPosition: boolean;
  isIncrease: boolean;
  isTrigger: boolean;
  isMarket: boolean;
  isLimit: boolean;
}

export type TokensRatio = {
  ratio: BN;
  largestToken: Token;
  smallestToken: Token;
};

export type SwapStats = {
  marketAddress: string;
  tokenInAddress: string;
  tokenOutAddress: string;
  isWrap: boolean;
  isUnwrap: boolean;
  isOutLiquidity?: boolean;
  swapFeeAmount: BN;
  swapFeeUsd: BN;
  priceImpactDeltaUsd: BN;
  amountIn: BN;
  amountInAfterFees: BN;
  usdIn: BN;
  amountOut: BN;
  usdOut: BN;
};

export type SwapPathStats = {
  swapPath: string[];
  swapSteps: SwapStats[];
  targetMarketAddress?: string;
  totalSwapPriceImpactDeltaUsd: BN;
  totalSwapFeeUsd: BN;
  totalFeesDeltaUsd: BN;
  tokenInAddress: string;
  tokenOutAddress: string;
  usdOut: BN;
  amountOut: BN;
};

export type SwapAmounts = {
  amountIn: BN;
  usdIn: BN;
  amountOut: BN;
  usdOut: BN;
  priceIn: BN;
  priceOut: BN;
  swapPathStats: SwapPathStats | undefined;
  minOutputAmount: BN;
  uiFeeUsd?: BN;
};

export interface IncreaseAmounts {
  initialCollateralDeltaAmount: BN,
  sizeDeltaUsd: BN,
}

export interface IncreaseSwapParams {
  initialCollateralToken: TokenData,
  swapPath: Address[],
  swapTokens: Address[],
}
