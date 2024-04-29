import { SharedStates } from "../../types";
import { createSharedStatesSelector } from "../../utils";

export const selectPositionState = (state: SharedStates) => state.position;
export const selectIsPositionLoading = createSharedStatesSelector([selectPositionState], state => state.isLoading);
export const selectPositionInfos = createSharedStatesSelector([selectPositionState], state => state.positionInfos);
export const selectPositionList = createSharedStatesSelector([selectPositionInfos], infos => Object.values(infos));
