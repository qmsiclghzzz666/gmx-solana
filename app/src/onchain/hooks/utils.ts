import { useAnchorProvider } from "@/contexts/anchor";
import { createAssociatedTokenAccountInstruction, createCloseAccountInstruction, createSyncNativeInstruction, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { useConnection } from "@solana/wallet-adapter-react";
import { ConfirmOptions, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { useCallback, useMemo } from "react";
import useSWR, { useSWRConfig } from "swr";
import useSWRMutation, { MutationFetcher } from "swr/mutation";
import { filterBalances } from "../token";
import { BN } from "@coral-xyz/anchor";
import { WRAPPED_NATIVE_TOKEN_ADDRESS } from "@/config/tokens";
import { usePending } from "@/contexts/pending";
import { TranscationInfo } from "../types";
import { helperToast } from "@/utils/helperToast";
import { makeSendErrorContent } from "./makeSendErrorContent";
import { makeSendingContent } from "./makeSendingContent";
import { t } from "@lingui/macro";

export const useGenesisHash = () => {
  const connection = useConnection();
  const endpoint = connection.connection.rpcEndpoint;
  const { data } = useSWR(`genesis/${endpoint}`, async () => {
    console.debug("genesis hash updated");
    return await connection.connection.getGenesisHash();
  }, {
    refreshInterval: 0,
  });

  return data;
};

export interface TriggerOptions {
  onSuccess?: () => void,
  onError?: () => void,
  disableSendingToast?: boolean,
  disableErrorToast?: boolean,
}

export const useTriggerInvocation = <T>(
  info: TranscationInfo,
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

export const useSendTransaction = <T>(
  info: TranscationInfo,
  makeTx: (arg: T, owner: PublicKey) => Transaction,
  opts?: ConfirmOptions & TriggerOptions,
) => {
  const provider = useAnchorProvider();
  const invoke = useCallback(async (arg: T) => {
    if (provider && provider.publicKey) {
      const commitment = opts?.commitment ?? "processed";
      const tx = makeTx(arg, provider.publicKey);
      tx.recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
      const signature = await provider.sendAndConfirm(tx, undefined, {
        ...opts,
        commitment,
      });
      return signature;
    } else {
      throw Error("Wallet is not connected");
    }
  }, [makeTx, opts, provider]);
  return useTriggerInvocation(info, invoke, opts)
};

export const useInitializeTokenAccount = (opts: ConfirmOptions = {
  commitment: "confirmed",
  preflightCommitment: "processed",
}) => {
  const { mutate } = useSWRConfig();

  return useSendTransaction({
    key: "init-token-account",
    onSentMessage: t`Initializing token account...`,
    message: t`Initialized token account.`,
  }, (token: PublicKey, owner) => {
    const address = getAssociatedTokenAddressSync(token, owner);
    const ix = createAssociatedTokenAccountInstruction(owner, address, owner, token);
    return new Transaction().add(ix);
  }, {
    onSuccess: () => {
      void mutate(filterBalances);
    },
    ...opts,
  });
};

export const useWrapNativeToken = (callback: () => void) => {
  const { mutate } = useSWRConfig();
  return useSendTransaction({
    key: "wrap-native-token",
    onSentMessage: t`Wrapping SOL...`,
    message: t`Wrapped SOL.`,
  }, (amount: BN, owner) => {
    const address = getAssociatedTokenAddressSync(WRAPPED_NATIVE_TOKEN_ADDRESS, owner);
    const tx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: owner,
        toPubkey: address,
        lamports: BigInt(amount.toString()),
      }),
      createSyncNativeInstruction(address),
    );
    return tx;
  }, {
    onSuccess: () => {
      callback();
      void mutate(filterBalances);
    },
    onError: () => {
      callback();
    }
  });
};

export const useUnwrapNativeToken = (callback: () => void) => {
  const { mutate } = useSWRConfig();
  return useSendTransaction({
    key: "unwrap-native-token",
    onSentMessage: t`Unwrapping WSOL...`,
    message: t`Unwrapped WSOL.`,
  }, (_arg: undefined, owner) => {
    const address = getAssociatedTokenAddressSync(WRAPPED_NATIVE_TOKEN_ADDRESS, owner);
    return new Transaction().add(
      createCloseAccountInstruction(address, owner, owner),
    );
  }, {
    onSuccess: () => {
      callback();
      void mutate(filterBalances);
    },
    onError: () => {
      callback();
    }
  });
};
