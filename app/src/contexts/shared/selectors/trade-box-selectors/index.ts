import { selectTradeBoxAvailableTokensOptions, selectTradeBoxFromTokenAddress, selectTradeBoxState, selectTradeBoxToTokenAddress, selectTradeBoxTradeFlags } from "@/contexts/shared/selectors/trade-box-selectors";
import { Token, TokenData, getTokenData } from "@/onchain/token";
import { convertToUsd, parseValue } from "@/utils/number";
import { createSharedStatesSelector } from "@/contexts/shared/utils";
import { selectMarketStateMarketInfos, selectMarketStateTokens } from "@/contexts/shared/selectors/market-selectors";
import { BN_ZERO, ONE_USD, USD_DECIMALS } from "@/config/constants";
import { BN } from "@coral-xyz/anchor";
import { selectTradeBoxCollateralTokenAddress } from "@/contexts/shared/selectors/trade-box-selectors/select-trade-box-collateral-token-address";
import { TokensRatio } from "@/onchain/trade";
import { getMarkPrice } from "@/utils/price";
import { getByKey } from "@/utils/objects";

const parseAmount = (value: string, token?: Token) => (token ? parseValue(value || "0", token.decimals) : BN_ZERO) ?? BN_ZERO;
const calcUsd = (amount: BN, token?: TokenData) => convertToUsd(amount, token?.decimals, token?.prices.minPrice);

export const selectFromToken = createSharedStatesSelector([selectMarketStateTokens, selectTradeBoxFromTokenAddress], getTokenData);
export const selectToToken = createSharedStatesSelector([selectMarketStateTokens, selectTradeBoxToTokenAddress], getTokenData);
export const selectCollateralToken = createSharedStatesSelector([selectMarketStateTokens, selectTradeBoxCollateralTokenAddress], getTokenData)
export const selectFromTokenInputValue = createSharedStatesSelector([selectTradeBoxState], state => state.fromTokenInputValue);
export const selectSetFromTokenInputValue = createSharedStatesSelector([selectTradeBoxState], state => state.setFromTokenInputValue);
export const selectToTokenInputValue = createSharedStatesSelector([selectTradeBoxState], state => state.toTokenInputValue);
export const selectSetToTokenInputValue = createSharedStatesSelector([selectTradeBoxState], state => state.setToTokenInputValue);
export const selectTriggerRatioInputValue = createSharedStatesSelector([selectTradeBoxState], state => state.triggerRatioInputValue);
export const selectSetTriggerRatioInputValue = createSharedStatesSelector([selectTradeBoxState], state => state.setTriggerRatioInputValue);
export const selectFocusedInput = createSharedStatesSelector([selectTradeBoxState], state => state.focusedInput);
export const selectSetFocusedInput = createSharedStatesSelector([selectTradeBoxState], state => state.setFocusedInput);
export const selectFromTokenInputAmount = createSharedStatesSelector([selectFromTokenInputValue, selectFromToken], parseAmount);
export const selectFromTokenUsd = createSharedStatesSelector([selectFromTokenInputAmount, selectFromToken], calcUsd);
export const selectToTokenInputAmount = createSharedStatesSelector([selectToTokenInputValue, selectToToken], parseAmount);
export const selectSwapTokens = createSharedStatesSelector([selectTradeBoxAvailableTokensOptions], options => options.swapTokens);
export const selectSortedLongAndShortTokens = createSharedStatesSelector([selectTradeBoxAvailableTokensOptions], options => options.sortedLongAndShortTokens);
export const selectSwitchTokenAddresses = createSharedStatesSelector([selectTradeBoxState], state => state.switchTokenAddresses);
export const selectTriggerRatioValue = createSharedStatesSelector([selectTriggerRatioInputValue], value => parseValue(value, USD_DECIMALS));
export const selectSortedAllMarkets = createSharedStatesSelector([selectTradeBoxAvailableTokensOptions], options => options.sortedAllMarkets);

export const selectMarketAddress = createSharedStatesSelector([selectTradeBoxState], state => state.marketAddress);
export const selectSetMarketAddress = createSharedStatesSelector([selectTradeBoxState], state => state.setMarketAddress);
export const selectStage = createSharedStatesSelector([selectTradeBoxState], state => state.stage);
export const selectSetStage = createSharedStatesSelector([selectTradeBoxState], state => state.setStage);

export const selectAvailableMarkets = createSharedStatesSelector([
  selectMarketStateMarketInfos,
  selectToToken,
  selectTradeBoxTradeFlags,
], (marketInfos, indexToken, { isPosition }) => {
  if (!isPosition || !indexToken) return [];
  return Object.values(marketInfos)
    .filter(market => !market.isDisabled && !market.isSpotOnly)
    .filter(market => market.indexTokenAddress.equals(indexToken.address));
});

export const selectMarketInfo = createSharedStatesSelector([selectMarketAddress, selectMarketStateMarketInfos], (
  marketAddress,
  marketInfos,
) => {
  return getByKey(marketInfos, marketAddress);
});

export const selectMarkPrice = createSharedStatesSelector([
  selectTradeBoxTradeFlags,
  selectToToken,
], ({ isSwap, isIncrease, isLong }, toToken) => {
  if (!toToken) return;
  if (isSwap) return toToken.prices.minPrice;
  return getMarkPrice({ prices: toToken.prices, isIncrease, isLong })
});

export const selectTradeRatios = createSharedStatesSelector([
  selectTradeBoxTradeFlags,
  selectFromToken,
  selectToToken,
  selectMarkPrice,
  selectTriggerRatioValue,
], ({ isSwap }, fromToken, toToken, markPrice, triggerRatioValue) => {
  const fromTokenPrice = fromToken?.prices.minPrice;

  if (!isSwap || !fromToken || !toToken || !fromTokenPrice || !markPrice) {
    return {};
  }

  const markRatio = getTokensRatioByPrice({
    fromToken,
    toToken,
    fromPrice: fromTokenPrice,
    toPrice: markPrice,
  });

  if (!triggerRatioValue) return { markPrice };

  const triggerRatio: TokensRatio = {
    ratio: triggerRatioValue.gt(BN_ZERO) ? triggerRatioValue : markRatio.ratio,
    largestToken: markRatio.largestToken,
    smallestToken: markRatio.smallestToken,
  };

  return {
    markRatio,
    triggerRatio,
  };
});

export const selectSwapRoutes = createSharedStatesSelector([
  selectMarketStateMarketInfos,
  selectTradeBoxFromTokenAddress,
  selectTradeBoxToTokenAddress,
  selectTradeBoxCollateralTokenAddress,
  selectTradeBoxTradeFlags,
], () => {
  // TODO: calculate swap routes.
  const findSwapPath = (usdIn: BN, opts: { byLiquidity?: boolean }) => {
    void opts;
    void usdIn;
    return undefined;
  };
  return {
    findSwapPath,
  };
});

// export const selectSwapAmounts = createSharedStatesSelector([
//   selectTradeBoxTradeFlags,
//   selectFromToken,
//   selectFromTokenInputAmount,
//   selectToToken,
//   selectToTokenInputAmount,
//   selectSwapRoutes,
//   selectTradeRatios,
//   selectFocusedInput,
// ], (
//   { isLimit },
//   fromToken,
//   fromTokenAmount,
//   toToken,
//   toTokenAmount,
//   { findSwapPath },
//   { markRatio, triggerRatio },
//   amountBy,
// ) => {
//   const fromTokenPrice = fromToken?.prices.minPrice;

//   if (!fromToken || !toToken || !fromTokenPrice) return;

//   if (amountBy === "from") {
//     return getSwapAmountsByFromValue({
//       tokenIn: fromToken,
//       tokenOut: toToken,
//       amountIn: fromTokenAmount,
//       triggerRatio: triggerRatio || markRatio,
//       isLimit,
//       findSwapPath,
//       // uiFeeFactor,
//     });
//   } else {
//     return getSwapAmountsByToValue({
//       tokenIn: fromToken,
//       tokenOut: toToken,
//       amountOut: toTokenAmount,
//       triggerRatio: triggerRatio || markRatio,
//       isLimit,
//       findSwapPath,
//       // uiFeeFactor,
//     });
//   }
// });

function getTokensRatioByPrice(p: {
  fromToken: TokenData;
  toToken: TokenData;
  fromPrice: BN;
  toPrice: BN;
}): TokensRatio {
  const { fromToken, toToken, fromPrice, toPrice } = p;

  const [largestToken, smallestToken, largestPrice, smallestPrice] = fromPrice.gt(toPrice)
    ? [fromToken, toToken, fromPrice, toPrice]
    : [toToken, fromToken, toPrice, fromPrice];

  const ratio = largestPrice.mul(ONE_USD).div(smallestPrice);

  return { ratio, largestToken, smallestToken };
}

export * from "./select-trade-box-state";
export * from "./select-trade-box-from-token-address";
export * from "./select-trade-box-set-from-token-address";
export * from "./select-trade-box-to-token-address";
export * from "./select-trade-box-trade-flags";
export * from "./select-trade-box-available-tokens-options";
export * from "./select-trade-box-trade-type";
export * from "./select-trade-box-choose-suitable-market";
export * from "./select-trade-box-get-max-long-short-liquidity-pool";
export * from "./select-trade-box-set-trade-params";
export * from "./select-trade-box-set-to-token-address";
export * from "./select-trade-box-trade-mode";
