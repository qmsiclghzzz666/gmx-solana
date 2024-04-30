import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

export const selectTradeBoxCollateralTokenAddress = createSharedStatesSelector([selectTradeBoxState], state => state.collateralAddress);
