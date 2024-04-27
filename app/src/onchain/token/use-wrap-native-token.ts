import { useSWRConfig } from "swr";
import { useSendTransaction } from "../transaction";
import { t } from "@lingui/macro";
import { BN } from "@coral-xyz/anchor";
import { createSyncNativeInstruction, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { SystemProgram, Transaction } from "@solana/web3.js";
import { WRAPPED_NATIVE_TOKEN_ADDRESS } from "@/config/tokens";
import { filterBalances } from "./use-tokens";

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
