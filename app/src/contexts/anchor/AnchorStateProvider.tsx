import { ReactNode, createContext, useMemo } from "react";
import { ConnectionProvider, WalletProvider, useAnchorWallet, useConnection } from "@solana/wallet-adapter-react";
import { Connection, PublicKey, clusterApiUrl } from "@solana/web3.js";
// import * as walletAdapterWallets from "@solana/wallet-adapter-wallets";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { AnchorProvider } from "@coral-xyz/anchor";
import { DEFAULT_CLUSTER } from "@/config/env";

export interface AnchorState {
  connection: Connection,
  active: boolean,
  owner?: PublicKey,
  provider?: AnchorProvider,
}

export const AnchorStateContext = createContext<AnchorState | null>(null);

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
  const wallets = useMemo(() => {
    return [
      // new walletAdapterWallets.SolflareWalletAdapter(),
      // new walletAdapterWallets.PhantomWalletAdapter(),
    ];
  }, []);

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect={true} onError={(e) => console.error("wallet error:", e)}>
        <WalletModalProvider>
          <Inner>
            {children}
          </Inner>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
