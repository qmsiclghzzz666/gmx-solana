import ExternalLink from "@/components/ExternalLink/ExternalLink";
import { getTransactionUrl } from "@/utils/transaction";
import { Trans, t } from "@lingui/macro";

export const makeSendErrorContent = (errorMessage: string | undefined) => function SendErrorContent() {
  const signature = extractSignatureFromError(errorMessage ?? "");
  return (
    <div>
      <Trans>Send Transaction Error.</Trans>
      <br />
      <br />
      <Trans>Error message: {errorMessage}</Trans>
      {signature && <>
        <br />
        <br />
        <ExternalLink href={getTransactionUrl(signature)}>{t`View the failed Tx`}</ExternalLink>
      </>}
    </div>
  );
};


function extractSignatureFromError(error: string): string | null {
  const match = error.match(/Raw transaction (.*) failed/);
  return match ? match[1] : null;
}
