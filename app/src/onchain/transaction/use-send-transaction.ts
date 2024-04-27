import { ConfirmOptions, PublicKey, Transaction } from "@solana/web3.js";
import { TransactionInfo } from "./types";
import { TriggerOptions, useTriggerInvocation } from "./use-trigger-invocation";
import { useAnchorProvider } from "@/contexts/anchor";
import { useCallback } from "react";

export const useSendTransaction = <T>(
  info: TransactionInfo,
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
  return useTriggerInvocation(info, invoke, opts);
};
