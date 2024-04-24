import { useAnchorProvider } from "@/contexts/anchor";
import { createAssociatedTokenAccountInstruction, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { useConnection } from "@solana/wallet-adapter-react";
import { PublicKey, Transaction } from "@solana/web3.js";
import { useCallback } from "react";
import useSWR, { useSWRConfig } from "swr";
import useSWRMutation from "swr/mutation";
import { filterBalances } from "../token";

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

export const useInitializeTokenAccount = () => {
  const provider = useAnchorProvider();
  const { mutate } = useSWRConfig();

  const { trigger } = useSWRMutation("init-token-account", async (_key, { arg }: { arg: PublicKey }) => {
    if (provider && provider.publicKey) {
      const address = getAssociatedTokenAddressSync(arg, provider.publicKey);
      const ix = createAssociatedTokenAccountInstruction(provider.publicKey, address, provider.publicKey, arg);
      const tx = new Transaction().add(ix);
      tx.recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
      await provider.sendAndConfirm(tx);
    } else {
      throw Error("Wallet not connected");
    }
  });

  return useCallback((token: PublicKey) => {
    void trigger(token, {
      onSuccess: () => {
        console.log(`token account for ${token.toBase58()} is initialized`);
        void mutate(filterBalances);
      },
      onError: (error) => {
        console.error(error)
      }
    });
  }, [trigger, mutate]);
};
