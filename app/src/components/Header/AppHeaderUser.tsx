import "./Header.scss";

import "@solana/wallet-adapter-react-ui/styles.css";
import { useWallet } from "@solana/wallet-adapter-react";
import ConnectWalletButton from "../Common/ConnectWalletButton/ConnectWalletButton";
import AddressDropdown from "../AddressDropdown/AddressDropdown";
import { useCallback } from "react";
import { useOpenConnectModal } from "@/contexts/anchor";

interface Props {
  small?: boolean;
}

export function AppHeaderUser({ small }: Props) {
  const wallet = useWallet();
  const { connected, connecting, disconnecting, publicKey } = wallet;
  const openConnectModal = useOpenConnectModal();
  const disconnectAccountAndCloseSettings = useCallback(() => {
    void wallet.disconnect();
  }, [wallet]);
  return (
    <div className="App-header-user">
      {/* <div className={cx("App-header-trade-link")}>
        <HeaderLink className="default-btn" to="/trade">
          <Trans>Trade</Trans>
        </HeaderLink>
      </div> */}
      <>
        {(connected && publicKey) ? (
          <div className="App-header-user-address">
            <AddressDropdown
              account={publicKey}
              disconnectAccountAndCloseSettings={disconnectAccountAndCloseSettings}
            />
          </div>
        ) : (
          <ConnectWalletButton
            small={small}
            onConnect={openConnectModal}
            onCancel={disconnectAccountAndCloseSettings}
            connecting={connecting}
            disconnecting={disconnecting}
          />
        )
        }
        {/* <NetworkDropdown
        small={small}
        networkOptions={NETWORK_OPTIONS}
        selectorLabel={selectorLabel}
        openSettings={openSettings}
      /> */}
      </>
    </div >
  );
}
