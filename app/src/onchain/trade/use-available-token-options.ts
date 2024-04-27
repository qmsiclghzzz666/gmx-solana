import { useMemo } from "react";
import { MarketInfo, MarketInfos } from "../market";
import { TokenData, Tokens, getMidPrice } from "../token";
import { AvailableTokenOptions } from "./types";
import { getByKey } from "@/utils/objects";
import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";
import { BN } from "@coral-xyz/anchor";
import { convertToUsd } from "@/utils/number";
import { BN_ZERO } from "@/config/constants";

export function useAvailableTokenOptions(
  {
    marketInfos,
    tokens
  }: {
    marketInfos?: MarketInfos,
    tokens?: Tokens,
  }): AvailableTokenOptions {
  return useMemo(() => {
    const marketsInfo = Object.values(marketInfos || {})
      .filter((market) => !market.isDisabled)
      .sort((a, b) => {
        return a.indexToken.symbol.localeCompare(b.indexToken.symbol);
      });
    const allMarkets = new Set<MarketInfo>();
    const nativeToken = getByKey(tokens, NATIVE_TOKEN_ADDRESS.toBase58());

    const indexTokens = new Set<TokenData>();
    const indexTokensWithPoolValue: { [address: string]: BN } = {};

    const collaterals = new Set<TokenData>();

    const longTokensWithPoolValue: { [address: string]: BN } = {};
    const shortTokensWithPoolValue: { [address: string]: BN } = {};

    for (const marketInfo of marketsInfo) {
      const longToken = marketInfo.longToken;
      const shortToken = marketInfo.shortToken;
      const indexToken = marketInfo.indexToken;

      if (marketInfo.isDisabled || !longToken || !shortToken || !indexToken) {
        continue;
      }

      if ((longToken.isWrapped || shortToken.isWrapped) && nativeToken) {
        collaterals.add(nativeToken);
      }

      collaterals.add(longToken);
      collaterals.add(shortToken);

      const longPoolAmountUsd = convertToUsd(
        marketInfo.longPoolAmount,
        marketInfo.longToken.decimals,
        getMidPrice(marketInfo.longToken.prices)
      )!;

      const shortPoolAmountUsd = convertToUsd(
        marketInfo.shortPoolAmount,
        marketInfo.shortToken.decimals,
        getMidPrice(marketInfo.shortToken.prices)
      )!;

      longTokensWithPoolValue[longToken.address.toBase58()] = (
        longTokensWithPoolValue[longToken.address.toBase58()] || BN_ZERO
      ).add(longPoolAmountUsd);

      shortTokensWithPoolValue[shortToken.address.toBase58()] = (
        shortTokensWithPoolValue[shortToken.address.toBase58()] || BN_ZERO
      ).add(shortPoolAmountUsd);

      if (!marketInfo.isSpotOnly) {
        indexTokens.add(indexToken);
        allMarkets.add(marketInfo);
        indexTokensWithPoolValue[indexToken.address.toBase58()] = (
          indexTokensWithPoolValue[indexToken.address.toBase58()] || BN_ZERO
        ).add(marketInfo.poolValueMax);
      }
    }

    const sortedIndexTokensWithPoolValue = Object.keys(indexTokensWithPoolValue).sort((a, b) => {
      return indexTokensWithPoolValue[b].gt(indexTokensWithPoolValue[a]) ? 1 : -1;
    });

    const sortedAllMarkets = Array.from(allMarkets).sort((a, b) => {
      return (
        sortedIndexTokensWithPoolValue.indexOf(a.indexToken.address.toBase58()) -
        sortedIndexTokensWithPoolValue.indexOf(b.indexToken.address.toBase58())
      );
    });

    const sortedLongTokens = Object.keys(longTokensWithPoolValue).sort((a, b) => {
      return longTokensWithPoolValue[b].gt(longTokensWithPoolValue[a]) ? 1 : -1;
    });

    const sortedShortTokens = Object.keys(shortTokensWithPoolValue).sort((a, b) => {
      return shortTokensWithPoolValue[b].gt(shortTokensWithPoolValue[a]) ? 1 : -1;
    });

    const sortedLongAndShortTokens = sortedLongTokens.concat(sortedShortTokens);

    return {
      tokens: tokens ?? {},
      swapTokens: Array.from(collaterals),
      indexTokens: Array.from(indexTokens),
      sortedIndexTokensWithPoolValue,
      sortedLongAndShortTokens: Array.from(new Set(sortedLongAndShortTokens)),
      sortedAllMarkets,
    };
  }, [marketInfos, tokens]);
}
