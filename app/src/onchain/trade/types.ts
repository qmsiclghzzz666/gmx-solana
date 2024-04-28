import { MarketInfo } from "../market";
import { TokenData, Tokens } from "../token";
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
    swapTokenAddress?: string,
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
