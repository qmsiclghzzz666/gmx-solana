import { Trans } from "@lingui/macro";
import cx from "classnames";

import "./Header.scss";
import { HeaderLink } from "./HeaderLink";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";

import "@solana/wallet-adapter-react-ui/styles.css";

interface Props {
  small?: boolean;
}

export function AppHeaderUser({ small }: Props) {
  void small;
  return (
    <div className="App-header-user">
      <div className={cx("App-header-trade-link")}>
        <HeaderLink className="default-btn" to="/trade">
          <Trans>Trade</Trans>
        </HeaderLink>
      </div>
      <>
        <WalletMultiButton />
        {/* <NetworkDropdown
          small={small}
          networkOptions={NETWORK_OPTIONS}
          selectorLabel={selectorLabel}
          openSettings={openSettings}
        /> */}
      </>
    </div>
  );
}
