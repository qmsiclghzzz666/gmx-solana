import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxFromTokenAddress, selectTradeBoxToTokenAddress, selectTradeBoxTradeFlags, selectTradeBoxAvailableTokensOptions } from "../trade-box-selectors";

export const selectAvailableChartTokens = createSharedStatesSelector(
  [
    selectTradeBoxFromTokenAddress,
    selectTradeBoxToTokenAddress,
    selectTradeBoxAvailableTokensOptions,
    selectTradeBoxTradeFlags,
  ],
  (
    fromTokenAddress,
    toTokenAddress,
    {
      swapTokens,
      indexTokens,
      sortedIndexTokensWithPoolValue,
      sortedLongAndShortTokens,
    },
    { isSwap },
  ) => {
    if (!fromTokenAddress || !toTokenAddress) {
      return [];
    }

    const availableChartTokens = isSwap ? swapTokens : indexTokens;
    const currentSortReferenceList = isSwap ? sortedLongAndShortTokens : sortedIndexTokensWithPoolValue;
    const sortedAvailableChartTokens = availableChartTokens.sort((a, b) => {
      if (currentSortReferenceList) {
        return currentSortReferenceList.indexOf(a.address.toBase58()) - currentSortReferenceList.indexOf(b.address.toBase58());
      } else {
        return 0;
      }
    });

    return sortedAvailableChartTokens;
  },
);
