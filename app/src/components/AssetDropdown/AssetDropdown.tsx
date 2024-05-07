import { Menu } from "@headlessui/react";
import "./AssetDropdown.scss";
import { FiChevronDown } from "react-icons/fi";
import cx from "classnames";
import { Token } from "@/onchain/token";
import ExternalLink from "../ExternalLink/ExternalLink";
import icon_solana from "@/img/ic_solana_24.svg";
import { Trans } from "@lingui/macro";
import { getAddressUrl } from "@/utils/explorer";

type Props = {
  assetSymbol?: string;
  token?: Token;
  position?: "left" | "right";
};

export function AssetDropdown({ assetSymbol, token, position = "right" }: Props) {
  if (!token || token.isNative || token.isSynthetic) return null;

  return (
    <div className="AssetDropdown-wrapper">
      <Menu>
        <Menu.Button as="div" className="dropdown-arrow center-both">
          <FiChevronDown size={20} />
        </Menu.Button>
        <Menu.Items as="div" className={cx("asset-menu-items", { left: position === "left" })}>
          <Menu.Item as="div">
            {!token.isNative && !token.isSynthetic && token.address && (
              <ExternalLink href={getAddressUrl(token.address)} className="asset-item">
                <img className="asset-item-icon" src={icon_solana} alt="Open in explorer" />
                <p>
                  <Trans>Open {assetSymbol ?? token.symbol} in Explorer</Trans>
                </p>
              </ExternalLink>
            )}
          </Menu.Item>
        </Menu.Items>
      </Menu>
    </div>
  );
}
