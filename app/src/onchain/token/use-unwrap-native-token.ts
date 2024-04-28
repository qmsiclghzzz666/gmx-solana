import { useSWRConfig } from "swr";
import { useSendTransaction } from "../transaction";
import { t } from "@lingui/macro";
import { createCloseAccountInstruction, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { Transaction } from "@solana/web3.js";
import { filterBalances } from "./use-tokens";
import { WRAPPED_NATIVE_TOKEN_ADDRESS } from "@/config/tokens";

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
