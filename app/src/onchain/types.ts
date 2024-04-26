export interface TranscationInfo {
  key: string,
  onSentMessage?: string,
  message: string,
  messageDetail?: string,
}

export type PendingTranscation = TranscationInfo & {
  signature: string,
};
