import { PendingTransaction } from "@/onchain/transaction";
import { Dispatch, SetStateAction } from "react";

export type PendingTxsSetter = Dispatch<SetStateAction<PendingTransaction[]>>;

export interface PendingTransactionsState {
  pendingTxs: PendingTransaction[],
  setPendingTxs: PendingTxsSetter,
}
