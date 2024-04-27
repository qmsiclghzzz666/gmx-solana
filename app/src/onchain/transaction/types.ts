export interface TransactionInfo {
  key: string,
  onSentMessage?: string,
  message: string,
  messageDetail?: string,
}

export type PendingTransaction = TransactionInfo & {
  signature: string,
};
