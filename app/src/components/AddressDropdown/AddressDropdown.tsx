import { Menu } from "@headlessui/react";
import { Trans } from "@lingui/macro";
import copy from "@/img/ic_copy_20.svg";
import disconnect from "@/img/ic_sign_out_20.svg";
import { FaChevronDown } from "react-icons/fa";
import { createBreakpoint, useCopyToClipboard } from "react-use";
import "./AddressDropdown.scss";
import { PublicKey } from "@solana/web3.js";
import { useWallet } from "@solana/wallet-adapter-react";

type Props = {
  account: PublicKey;
  // accountUrl: string;
  disconnectAccountAndCloseSettings: () => void;
};

function shortenAddress(address: string, length: number, padStart: number = 1) {
  if (!length) {
    return "";
  }
  if (!address) {
    return address;
  }
  if (address.length < 10) {
    return address;
  }
  if (length >= address.length) {
    return address;
  }
  const left = Math.floor((length - 3) / 2) + (padStart || 0);
  return address.substring(0, left) + "..." + address.substring(address.length - (length - (left + 3)), address.length);
}

function AddressDropdown({ account, disconnectAccountAndCloseSettings }: Props) {
  const useBreakpoint = createBreakpoint({ L: 600, M: 550, S: 400 });
  const breakpoint = useBreakpoint();
  const [, copyToClipboard] = useCopyToClipboard();
  // const { ensName } = useENS(account);
  // const { provider: ethereumProvider } = useJsonRpcProvider(ETH_MAINNET);
  const displayAddressLength = breakpoint === "S" ? 9 : 13;
  // const [, setOneClickModalOpen] = useSubaccountModalOpen();
  // const handleSubaccountClick = useCallback(() => {
  //   setOneClickModalOpen(true);
  // }, [setOneClickModalOpen]);

  const { wallet } = useWallet();

  return (
    <Menu>
      <Menu.Button as="div">
        <button className="App-cta small transparent address-btn">
          <div className="user-avatar">
            {wallet && <img width={20} src={wallet.adapter.icon} />}
          </div>
          <span className="user-address">{shortenAddress(account.toBase58(), displayAddressLength)}</span>
          <FaChevronDown />
        </button>
      </Menu.Button>
      <div>
        <Menu.Items as="div" className="menu-items">
          <Menu.Item>
            <div
              className="menu-item"
              onClick={() => {
                copyToClipboard(account.toBase58());
                // helperToast.success(t`Address copied to your clipboard`);
              }}
            >
              <img width={20} src={copy} alt="Copy user address" />
              <p>
                <Trans>Copy Address</Trans>
              </p>
            </div>
          </Menu.Item>
          {/* <Menu.Item>
            <ExternalLink href={accountUrl} className="menu-item">
              <img width={20} src={externalLink} alt="Open address in explorer" />
              <p>
                <Trans>View in Explorer</Trans>
              </p>
            </ExternalLink>
          </Menu.Item> */}
          {/* <Menu.Item>
            <div className="menu-item" onClick={handleSubaccountClick}>
              <img width={20} src={oneClickTradingIcon} alt="Open One-click Trading settings" />
              <p>
                <Trans>One-Click Trading</Trans>
              </p>
            </div>
          </Menu.Item> */}
          <Menu.Item>
            <div className="menu-item" onClick={disconnectAccountAndCloseSettings}>
              <img width={20} src={disconnect} alt="Disconnect the wallet" />
              <p>
                <Trans>Disconnect</Trans>
              </p>
            </div>
          </Menu.Item>
        </Menu.Items>
      </div>
    </Menu>
  );
}

export default AddressDropdown;
