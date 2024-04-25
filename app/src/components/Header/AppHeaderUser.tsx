import { Trans } from "@lingui/macro";
import cx from "classnames";

import "./Header.scss";
import { HeaderLink } from "./HeaderLink";
import connectWalletImg from "@/img/ic_wallet_24.svg";

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
  const openConnectModal = useOpenConnectModal();
  const disconnectAccountAndCloseSettings = useCallback(() => {
    void wallet.disconnect();
  }, [wallet]);
  return (
    <div className="App-header-user">
      <div className={cx("App-header-trade-link")}>
        <HeaderLink className="default-btn" to="/trade">
          <Trans>Trade</Trans>
        </HeaderLink>
      </div>
      <>
        {(wallet.connected && wallet.publicKey) ? (
          <div className="App-header-user-address">
            <AddressDropdown
              account={wallet.publicKey}
              disconnectAccountAndCloseSettings={disconnectAccountAndCloseSettings}
            />
          </div>
        ) : (
          <ConnectWalletButton onClick={openConnectModal} imgSrc={connectWalletImg}>
            {small ? <Trans>Connect</Trans> : <Trans>Connect Wallet</Trans>}
          </ConnectWalletButton>
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
