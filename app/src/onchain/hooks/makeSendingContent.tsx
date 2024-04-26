import { TranscationInfo } from "../types";

export const makeSendingContent = (info: TranscationInfo) => function SendSuccessContent() {
  return (
    <div>
      {info.onSentMessage ? info.onSentMessage : info.message}
      {info.messageDetail && info.messageDetail}
    </div>
  );
};
