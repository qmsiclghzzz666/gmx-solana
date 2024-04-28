import { useSWRConfig } from "swr";
import { useSendTransaction } from "../transaction";
import { ConfirmOptions, PublicKey, Transaction } from "@solana/web3.js";
import { t } from "@lingui/macro";
import { createAssociatedTokenAccountInstruction, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { filterBalances } from "./use-tokens";

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
