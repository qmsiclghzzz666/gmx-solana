import { ReactNode, useMemo } from "react";
import { ConnectionProvider, WalletProvider, useAnchorWallet, useConnection } from "@solana/wallet-adapter-react";
import { clusterApiUrl } from "@solana/web3.js";
import * as walletAdapterWallets from "@solana/wallet-adapter-wallets";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { AnchorContext } from "../contexts/anchor";
import { AnchorProvider } from "@coral-xyz/anchor";

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
