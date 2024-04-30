import { getByKey } from "@/utils/objects";
import { SharedStates } from "../../types";
import { createSharedStatesSelector } from "../../utils";
import { selectPositionInfos } from "../position-selectors";

export const selectPositionSeller = (state: SharedStates) => state.positionSeller;
export const selectClosingPositionAddress = createSharedStatesSelector([selectPositionSeller], seller => seller.address);
export const selectSetClosingPositionAddress = createSharedStatesSelector([selectPositionSeller], seller => seller.setAddress);

export const selectClosingPosition = createSharedStatesSelector([
  selectPositionInfos,
  selectClosingPositionAddress
], (positions, address) => {
  return getByKey(positions, address?.toBase58());
});
