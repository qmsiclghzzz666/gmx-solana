import { ReactNode, createContext, useMemo } from "react";
import { ConnectionProvider, WalletProvider, useAnchorWallet, useConnection } from "@solana/wallet-adapter-react";
import { Connection, clusterApiUrl } from "@solana/web3.js";
import * as walletAdapterWallets from "@solana/wallet-adapter-wallets";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { AnchorProvider } from "@coral-xyz/anchor";

export interface AnchorContextType {
  connection: Connection | null,
  provider: AnchorProvider | null,
}

export const AnchorContext = createContext<AnchorContextType>({
  connection: null,
  provider: null,
});

function Inner({ children }: { children: ReactNode }) {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const provider = wallet ? new AnchorProvider(connection, wallet) : null;
  return (
    <AnchorContext.Provider value={{
      connection,
      provider,
    }}>
      {children}
    </AnchorContext.Provider>
  )
}

export function AnchorContextProvider({ children }: { children: ReactNode }) {
  const endpoint = clusterApiUrl("devnet");
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
