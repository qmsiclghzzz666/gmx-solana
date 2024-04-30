import { usePending } from "@/contexts/pending";
import { TransactionInfo } from "./types";
import useSWRMutation, { MutationFetcher } from "swr/mutation";
import { useCallback, useMemo } from "react";
import { helperToast } from "@/utils/helperToast";
import { makeSendErrorContent } from "./makeSendErrorContent";
import { makeSendingContent } from "./makeSendingContent";

export interface TriggerOptions {
  onSuccess?: () => void,
  onError?: () => void,
  disableSendingToast?: boolean,
  disableErrorToast?: boolean,
}

export const useTriggerInvocation = <T>(
  info: TransactionInfo,
  invoke: (arg: T) => Promise<string>,
  opts?: TriggerOptions,
) => {
  const { setPendingTxs } = usePending();
  const { key } = info;

  const fetcher: MutationFetcher<string, string, { arg: T }> = useCallback(async (_key, { arg: { arg } }) => {
    const signature = await invoke(arg);
    setPendingTxs((txs) => {
      return [...txs, {
        ...info,
        signature,
      }];
    });
    return signature;
  }, [info, invoke, setPendingTxs]);

  const { trigger, isMutating } = useSWRMutation<string, Error, string, { arg: T }, string>(key, fetcher, {
    throwOnError: false,
  });

  return useMemo(() => {
    return {
      isSending: isMutating,
      trigger: (arg: T) => {
        const res = trigger({ arg }, {
          onSuccess: () => {
            if (opts?.onSuccess) {
              opts.onSuccess();
            }
          },
          onError: (error: Error) => {
            console.error(error);
            if (!opts?.disableErrorToast) {
              helperToast.error(makeSendErrorContent(error));
            }
            if (opts?.onError) {
              opts.onError();
            }
          },
        });
        if (!opts?.disableSendingToast) {
          helperToast.info(makeSendingContent(info));
        }
        return res;
      }
    }
  }, [info, isMutating, opts, trigger]);
}
