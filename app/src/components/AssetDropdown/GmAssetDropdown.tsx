import { Menu } from "@headlessui/react";
import "./AssetDropdown.scss";
import "./GmAssetDropdown.scss";
import { FiChevronDown } from "react-icons/fi";
import cx from "classnames";
import ExternalLink from "../ExternalLink/ExternalLink";
import { Trans } from "@lingui/macro";
import icon_solana from "@/img/ic_solana_24.svg";
import { MarketInfo } from "@/onchain/market";
import { getAddressUrl } from "@/utils/explorer";
import { createBreakpoint } from "react-use";

type Props = {
  market: MarketInfo,
  position?: "left" | "right";
};

const useBreakpoint = createBreakpoint({ S: 0, M: 600 });

export function GmAssetDropdown({ market, position = "right" }: Props) {
  const breakpoint = useBreakpoint();

  return (
    <div className="AssetDropdown-wrapper GmAssetDropdown">
      <Menu>
        <Menu.Button as="div" className="dropdown-arrow center-both">
          <FiChevronDown size={20} />
        </Menu.Button>
        <Menu.Items
          as="div"
          className={cx("asset-menu-items", breakpoint === "S" ? "center" : { left: position === "left" })}
        >
          {market && (
            <Menu.Item as="div">
              <ExternalLink href={getAddressUrl(market.marketTokenAddress)} className="asset-item">
                <img className="asset-item-icon" src={icon_solana} alt="Open in explorer" />
                <p>
                  <Trans>Open {market.name} in Explorer</Trans>
                </p>
              </ExternalLink>
            </Menu.Item>
          )}
        </Menu.Items>
      </Menu>
    </div>
  );
}
