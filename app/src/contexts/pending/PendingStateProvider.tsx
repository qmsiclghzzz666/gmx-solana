import { ReactNode, useEffect, useMemo, useState } from "react";
import { PendingTransactionsStateContext } from ".";
import useSWR from "swr";
import { useConnection } from "@solana/wallet-adapter-react";
import { helperToast } from "@/utils/helperToast";
import { Trans } from "@lingui/macro";
import ExternalLink from "@/components/ExternalLink/ExternalLink";
import { getTransactionUrl } from "@/utils/transaction";
import { PendingTransaction } from "@/onchain/transaction";

interface Props {
  children: ReactNode,
}

export function PendingStateProvider({
  children,
}: Props) {
  const [pendingTxs, setPendingTxs] = useState<PendingTransaction[]>([]);

  const request = useMemo(() => {
    return {
      key: "check_txs",
      pendingTxs: pendingTxs.map(tx => tx.signature),
    }
  }, [pendingTxs]);

  const { connection } = useConnection();

  const { data } = useSWR(request, async ({ pendingTxs }) => {
    try {
      return (await connection.getSignatureStatuses(pendingTxs)).value ?? [];
    } catch (e) {
      const error = e as Error;
      helperToast.error(
        <div>
          <Trans>
            Failed to get signature statuses.
          </Trans>
          <br />
          {error.message}
        </div>
      );
      return [];
    }
  }, {
    refreshInterval: 2000,
  });

  useEffect(() => {
    if (data) {
      const updatedPendingTxs: PendingTransaction[] = [];
      for (let i = 0; i < pendingTxs.length; i++) {
        const pendingTx = pendingTxs[i];
        const status = data[i];
        if (status) {
          if (status.confirmationStatus === "confirmed" || status.confirmationStatus === "finalized") {
            const url = getTransactionUrl(pendingTx.signature);
            if (status.err) {
              helperToast.error(
                <div>
                  <Trans>
                    Tx failed. <ExternalLink href={url}>View</ExternalLink>
                  </Trans>
                </div>
              );
            } else {
              helperToast.success(
                <div>
                  {pendingTx.message}{" "}
                  <ExternalLink href={url}>
                    <Trans>View Tx</Trans>
                  </ExternalLink>
                </div>
              );
            }
            continue;
          }
        } else {
          helperToast.info(
            <div>
              <Trans>
                {`Status for tx ${pendingTx.signature} not found`}
              </Trans>
            </div>
          );
        }
        updatedPendingTxs.push(pendingTx);
      }
      if (updatedPendingTxs.length !== pendingTxs.length) {
        setPendingTxs(updatedPendingTxs);
      }
    }
  }, [data, pendingTxs]);

  const value = useMemo(() => {
    return {
      pendingTxs,
      setPendingTxs,
    }
  }, [pendingTxs]);
  return (
    <PendingTransactionsStateContext.Provider value={value}>
      {children}
    </PendingTransactionsStateContext.Provider>
  );
}
