import { getAvailableUsdLiquidityForPosition } from "@/onchain/market";
import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";
import { bnClampMin, toBigInt } from "@/utils/number";
import { BN_ZERO } from "@/config/constants";
import { groupBy, maxBy } from "lodash";
import { Token, TokenOption } from "@/onchain/token";

const selectSortedAllMarkets = createSharedStatesSelector([selectTradeBoxState], state => state.avaiableTokensOptions.sortedAllMarkets);

export const selectTradeBoxGetMaxLongShortLiquidityPool = createSharedStatesSelector([
  selectSortedAllMarkets,
], (sortedAllMarkets) => {

  const marketsWithMaxReservedUsd = sortedAllMarkets.map((marketInfo) => {
    const maxLongLiquidity = getAvailableUsdLiquidityForPosition(marketInfo, true);
    const maxShortLiquidity = getAvailableUsdLiquidityForPosition(marketInfo, false);

    return {
      maxLongLiquidity: bnClampMin(maxLongLiquidity, BN_ZERO),
      maxShortLiquidity: bnClampMin(maxShortLiquidity, BN_ZERO),
      marketTokenAddress: marketInfo.marketTokenAddress.toBase58(),
      indexTokenAddress: marketInfo.indexTokenAddress.toBase58(),
    };
  });

  const groupedIndexMarkets: { [marketAddress: string]: TokenOption[] } = groupBy(
    marketsWithMaxReservedUsd,
    (market) => market.indexTokenAddress
  );

  return (token: Token) => {
    const indexTokenAddress = token.isNative ? token.wrappedAddress : token.address;
    const currentMarkets = groupedIndexMarkets[indexTokenAddress!.toBase58()];
    const maxLongLiquidityPool = maxBy(currentMarkets, (market) => toBigInt(market.maxLongLiquidity))!;
    const maxShortLiquidityPool = maxBy(currentMarkets, (market) => toBigInt(market.maxShortLiquidity))!;

    return {
      maxLongLiquidityPool,
      maxShortLiquidityPool,
    };
  };
});
