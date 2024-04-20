import { ReactNode, createContext, useMemo } from "react";
import { ConnectionProvider, WalletProvider, useAnchorWallet, useConnection } from "@solana/wallet-adapter-react";
import { Connection, clusterApiUrl } from "@solana/web3.js";
import * as walletAdapterWallets from "@solana/wallet-adapter-wallets";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { AnchorProvider } from "@coral-xyz/anchor";
import { DEFAULT_CLUSTER } from "@/config/env";

export interface AnchorContext {
  connection: Connection,
  provider?: AnchorProvider,
}

export const AnchorContextCtx = createContext<AnchorContext | null>(null);

function Inner({ children }: { children: ReactNode }) {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const provider = wallet ? new AnchorProvider(connection, wallet) : undefined;
  return (
    <AnchorContextCtx.Provider value={{
      connection,
      provider,
    }}>
      {children}
    </AnchorContextCtx.Provider>
  )
}

export function AnchorContextProvider({ children }: { children: ReactNode }) {
  const endpoint = clusterApiUrl(DEFAULT_CLUSTER);
  const wallets = useMemo(() => {
    return [
      new walletAdapterWallets.PhantomWalletAdapter(),
    ];
  }, []);

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect={true}>
        <WalletModalProvider>
          <Inner>
            {children}
          </Inner>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
