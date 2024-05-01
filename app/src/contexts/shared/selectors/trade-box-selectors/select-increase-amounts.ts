import { IncreaseAmounts } from "@/onchain/trade";
import { selectFromTokenInputAmount, selectToToken, selectToTokenInputAmount, selectTradeBoxTradeFlags } from ".";
import { createSharedStatesSelector } from "../../utils";
import { convertToUsd } from "@/utils/number";
import { getMidPrice } from "@/onchain/token";
import { BN_ZERO } from "@/config/constants";

export const selectIncreaseAmounts = createSharedStatesSelector([
  selectTradeBoxTradeFlags,
  selectFromTokenInputAmount,
  selectToTokenInputAmount,
  selectToToken,
], (
  { isIncrease },
  initialCollateralDeltaAmount,
  indexTokenAmount,
  indexToken,
) => {
  if (!isIncrease || !indexToken) return;
  return {
    initialCollateralDeltaAmount,
    sizeDeltaUsd: convertToUsd(indexTokenAmount, indexToken.decimals, getMidPrice(indexToken.prices)) ?? BN_ZERO,
  } satisfies IncreaseAmounts
});
