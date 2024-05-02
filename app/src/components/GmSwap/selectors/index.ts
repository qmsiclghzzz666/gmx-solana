import { createSelector, createStructuredSelector } from "reselect"
import { GmState, Operation } from "../types";
import { TokenData } from "@/onchain/token";
import { convertToUsd, parseValue } from "@/utils/number";
import { BN } from "@coral-xyz/anchor";
import { BN_ZERO } from "@/config/constants";

export const createGmSelector = createSelector.withTypes<GmState>();

const parseInputValue = (value: string, token?: TokenData) => parseValue(value, token?.decimals || 0) ?? BN_ZERO;
const calcUsd = (isBuy: boolean) => (operation: Operation, amount: BN, token?: TokenData) => convertToUsd(
  amount,
  token?.decimals,
  operation === Operation.Deposit ? (isBuy ? token?.prices?.minPrice : token?.prices?.maxPrice) : (isBuy ? token?.prices?.maxPrice : token?.prices?.minPrice)
);

export const selectMarket = (state: GmState) => state.market;
export const selectInputState = (state: GmState) => state.input;
export const selectFirstToken = (state: GmState) => state.firstToken;
export const selectSecondToken = (state: GmState) => state.secondToken;
export const selectMarketToken = (state: GmState) => state.marketToken;
export const selectOperation = (state: GmState) => state.operation;
export const selectFirstInputValue = createGmSelector([selectInputState], input => input.firstTokenInputValue);
export const selectSecondInputValue = createGmSelector([selectInputState], input => input.secondTokenInputValue);
export const selectMarketInputValue = createGmSelector([selectInputState], input => input.marketTokenInputValue);
export const selectFirstTokenAmount = createGmSelector([selectFirstInputValue, selectFirstToken], parseInputValue);
export const selectSecondTokenAmount = createGmSelector([selectSecondInputValue, selectSecondToken], parseInputValue);
export const selectMarketTokenAmount = createGmSelector([selectMarketInputValue, selectMarketToken], parseInputValue);
export const selectFirstTokenUsd = createGmSelector([selectOperation, selectFirstTokenAmount, selectFirstToken], calcUsd(true));
export const selectSecondTokenUsd = createGmSelector([selectOperation, selectSecondTokenAmount, selectSecondToken], calcUsd(true));
export const selectMarketTokenUsd = createGmSelector([selectOperation, selectMarketTokenAmount, selectMarketToken], calcUsd(false));
export const selectIsDeposit = createGmSelector([selectOperation], operation => operation === Operation.Deposit);

export const selectInputAmounts = createStructuredSelector({
  firstTokenAmount: selectFirstTokenAmount,
  secondTokenAmount: selectSecondTokenAmount,
  marketTokenAmount: selectMarketTokenAmount,
}, createGmSelector);

export const selectInputDisplay = createStructuredSelector({
  firstTokenUsd: selectFirstTokenUsd,
  secondTokenUsd: selectSecondTokenUsd,
  marketTokenUsd: selectMarketTokenUsd,
}, createGmSelector);

export const selectTokens = createStructuredSelector({
  firstToken: selectFirstToken,
  secondToken: selectSecondToken,
  marketToken: selectMarketToken,
}, createGmSelector);

export const selectParams = createStructuredSelector({
  tokens: selectTokens,
  amounts: selectInputAmounts,
  display: selectInputDisplay,
}, createGmSelector);
