import { Dispatch, SetStateAction } from "react";

export interface PendingTransactionsState {
  pendingTxs: string[],
  setPendingTxs: Dispatch<SetStateAction<string[]>>,
}
