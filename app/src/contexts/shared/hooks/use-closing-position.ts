import { selectClosingPosition } from "../selectors/position-seller-selectors";
import { useSharedStatesSelector } from "./use-shared-states-selector";

export const useClosingPosition = () => useSharedStatesSelector(selectClosingPosition);
