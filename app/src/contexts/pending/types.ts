import { PendingTranscation } from "@/onchain";
import { Dispatch, SetStateAction } from "react";

export interface PendingTransactionsState {
  pendingTxs: PendingTranscation[],
  setPendingTxs: Dispatch<SetStateAction<PendingTranscation[]>>,
}
