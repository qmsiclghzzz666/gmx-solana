import { ReactNode, useMemo } from "react";
import { ConnectionProvider, WalletProvider, useAnchorWallet, useConnection } from "@solana/wallet-adapter-react";
import { clusterApiUrl } from "@solana/web3.js";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { AnchorProvider } from "@coral-xyz/anchor";
import { DEFAULT_CLUSTER } from "@/config/env";
import { AnchorStateContext } from ".";

function Inner({ children }: { children: ReactNode }) {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const value = useMemo(() => {
    const provider = wallet ? new AnchorProvider(connection, wallet) : undefined;
    return {
      connection,
      provider,
      active: Boolean(provider && provider.publicKey),
      owner: provider?.publicKey,
    }
  }, [connection, wallet]);
  return (
    <AnchorStateContext.Provider value={value}>
      {children}
    </AnchorStateContext.Provider>
  )
}

export function AnchorStateProvider({ children }: { children: ReactNode }) {
  const endpoint = clusterApiUrl(DEFAULT_CLUSTER);
  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={[]} autoConnect={true} onError={(e) => console.error("wallet error:", e)}>
        <WalletModalProvider>
          <Inner>
            {children}
          </Inner>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
