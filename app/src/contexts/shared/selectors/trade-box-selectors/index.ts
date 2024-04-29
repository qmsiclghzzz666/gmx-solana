import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

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

export const selectMarketAddress = createSharedStatesSelector([selectTradeBoxState], state => state.marketAddress);
