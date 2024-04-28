import "./ChartTokenSelector.scss";

import { Token } from "@/onchain/token";
import { Popover } from "@headlessui/react";
import TokenIcon from "../TokenIcon/TokenIcon";
import { FaChevronDown } from "react-icons/fa";
import SearchInput from "../SearchInput/SearchInput";
import cx from "classnames";
import { Trans, t } from "@lingui/macro";
import { formatUsd, getMarketIndexName, getMarketPoolName } from "../MarketsList/utils";
import { useCallback, useState } from "react";
import { useMarketInfos, useSharedStatesSelector } from "@/contexts/shared";
import { selectTradeBoxChooseSuitableMarket, selectTradeBoxGetMaxLongShortLiquidityPool, selectTradeBoxTradeFlags } from "@/contexts/shared/selectors/trade-box-selectors";
import { PreferredTradeTypePickStrategy } from "@/onchain/market";
import { TradeType } from "@/onchain/trade";
import { getByKey } from "@/utils/objects";
import { helperToast } from "@/utils/helperToast";

type Props = {
  selectedToken: Token | undefined;
  options: Token[] | undefined;
};

export function ChartTokenSelector({ options, selectedToken }: Props) {
  const [searchKeyword, setSearchKeyword] = useState("");
  const { isSwap } = useSharedStatesSelector(selectTradeBoxTradeFlags);

  const filteredTokens: Token[] | undefined = options?.filter((item) => {
    return (
      // item.name.toLowerCase().indexOf(searchKeyword.toLowerCase()) > -1 ||
      item.symbol.toLowerCase().indexOf(searchKeyword.toLowerCase()) > -1
    );
  });

  const chooseSuitableMarket = useSharedStatesSelector(selectTradeBoxChooseSuitableMarket);
  const marketsInfoData = useMarketInfos();

  const handleMarketSelect = useCallback(
    (tokenAddress: string, preferredTradeType?: PreferredTradeTypePickStrategy) => {
      setSearchKeyword("");
      const chosenMarket = chooseSuitableMarket(tokenAddress, preferredTradeType);

      if (chosenMarket?.marketTokenAddress && chosenMarket.tradeType !== TradeType.Swap) {
        const marketInfo = getByKey(marketsInfoData, chosenMarket.marketTokenAddress);
        const nextTradeType = chosenMarket.tradeType;
        if (marketInfo) {
          const indexName = getMarketIndexName(marketInfo);
          const poolName = getMarketPoolName(marketInfo);
          helperToast.info(
            <Trans>
              <span>{nextTradeType === TradeType.Long ? t`Long` : t`Short`}</span>{" "}
              <div className="inline-flex">
                <span>{indexName}</span>
                <span className="subtext gm-toast lh-1">[{poolName}]</span>
              </div>{" "}
              <span>market selected</span>
            </Trans>
          );
        }
      }
    },
    [chooseSuitableMarket, marketsInfoData]
  );

  const getMaxLongShortLiquidityPool = useSharedStatesSelector(selectTradeBoxGetMaxLongShortLiquidityPool);

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
                      handleMarketSelect(token.address.toBase58());
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
                              onClick={() => handleMarketSelect(token.address.toBase58(), "largestPosition")}
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
                                handleMarketSelect(token.address.toBase58(), TradeType.Long);
                              }}
                            >
                              {!isSwap && maxLongLiquidityPool ? formatUsd(maxLongLiquidityPool?.maxLongLiquidity) : ""}
                            </td>
                            <td
                              onClick={() => {
                                handleMarketSelect(token.address.toBase58(), TradeType.Short);
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
