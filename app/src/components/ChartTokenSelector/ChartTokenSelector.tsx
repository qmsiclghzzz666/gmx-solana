import "./ChartTokenSelector.scss";

import { Token } from "@/onchain/token";
import { Popover } from "@headlessui/react";
import TokenIcon from "../TokenIcon/TokenIcon";
import { FaChevronDown } from "react-icons/fa";
import SearchInput from "../SearchInput/SearchInput";
import cx from "classnames";
import { Trans } from "@lingui/macro";
import { formatUsd } from "../MarketsList/utils";

type Props = {
  selectedToken: Token | undefined;
  options: Token[] | undefined;
};

export function ChartTokenSelector(props: Props) {
  return (
    <Popover className="Synths-ChartTokenSelector">
      {({ open, close }) => {
        if (!open && searchKeyword.length > 0) setSearchKeyword("");
        return (
          <>
            <Popover.Button as="div">
              <button className={cx("chart-token-selector", { "chart-token-label--active": open })}>
                {selectedToken && (
                  <span className="chart-token-selector--current inline-items-center">
                    <TokenIcon
                      className="chart-token-current-icon"
                      symbol={selectedToken.symbol}
                      displaySize={20}
                      importSize={24}
                    />
                    {selectedToken.symbol} {"/ USD"}
                  </span>
                )}
                <FaChevronDown fontSize={14} />
              </button>
            </Popover.Button>
            <div className="chart-token-menu">
              <Popover.Panel as="div" className={cx("menu-items chart-token-menu-items", { isSwap: isSwap })}>
                <SearchInput
                  className="m-md"
                  value={searchKeyword}
                  setValue={({ target }) => setSearchKeyword(target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && filteredTokens && filteredTokens.length > 0) {
                      const token = filteredTokens[0];
                      handleMarketSelect(token.address);
                      close();
                    }
                  }}
                />
                <div className="divider" />
                <div className="chart-token-list">
                  <table>
                    {filteredTokens && filteredTokens.length > 0 && (
                      <thead className="table-head">
                        <tr>
                          <th>
                            <Trans>Market</Trans>
                          </th>
                          <th>{!isSwap && t`LONG LIQ.`}</th>
                          <th>{!isSwap && t`SHORT LIQ.`}</th>
                        </tr>
                      </thead>
                    )}
                    <tbody>
                      {filteredTokens?.map((token) => {
                        const { maxLongLiquidityPool, maxShortLiquidityPool } = getMaxLongShortLiquidityPool(token);
                        return (
                          <Popover.Button
                            as="tr"
                            key={token.symbol}
                            className={isSwap ? "Swap-token-list" : "Position-token-list"}
                          >
                            <td
                              className="token-item"
                              onClick={() => handleMarketSelect(token.address, "largestPosition")}
                            >
                              <span className="inline-items-center">
                                <TokenIcon
                                  className="ChartToken-list-icon"
                                  symbol={token.symbol}
                                  displaySize={16}
                                  importSize={24}
                                />
                                {token.symbol} {!isSwap && "/ USD"}
                              </span>
                            </td>

                            <td
                              onClick={() => {
                                handleMarketSelect(token.address, TradeType.Long);
                              }}
                            >
                              {!isSwap && maxLongLiquidityPool ? formatUsd(maxLongLiquidityPool?.maxLongLiquidity) : ""}
                            </td>
                            <td
                              onClick={() => {
                                handleMarketSelect(token.address, TradeType.Short);
                              }}
                            >
                              {!isSwap && maxShortLiquidityPool
                                ? formatUsd(maxShortLiquidityPool?.maxShortLiquidity)
                                : ""}
                            </td>
                          </Popover.Button>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              </Popover.Panel>
            </div>
          </>
        );
      }}
    </Popover>
  );
}
