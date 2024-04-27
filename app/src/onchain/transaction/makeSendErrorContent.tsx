import { Trans } from "@lingui/macro";

export const makeSendErrorContent = (error: Error) => function SendErrorContent() {
  return (
    <div>
      <Trans>Send Transaction Error:</Trans>
      <br />
      {error.message}
    </div>
  );
};
