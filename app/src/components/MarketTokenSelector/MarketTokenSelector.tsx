import "./MarketTokenSelector.scss";
import { useMemo, useState } from "react";
import { Popover } from "@headlessui/react";
import cx from "classnames";
import { FaChevronDown } from "react-icons/fa";
import { MarketInfo, MarketInfos, MarketTokenAPRs } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { useSortedPoolsWithIndexToken } from "@/hooks";
import { getMarketIndexName, getMarketPoolName } from "../MarketsList/utils";
import { getByKey } from "@/utils/objects";
import { getSellableMarketToken } from "@/onchain/market/utils";
import { useNavigate } from "react-router-dom";
import { getNormalizedTokenSymbol } from "@/utils/tokens";
import TokenIcon from "../TokenIcon/TokenIcon";
import { formatTokenAmount } from "@/utils/number";
import SearchInput from "../SearchInput/SearchInput";
import { BN_ZERO } from "@/config/constants";

type Props = {
  marketsInfoData?: MarketInfos;
  marketTokensData?: Tokens;
  marketsTokensAPRData?: MarketTokenAPRs;
  marketsTokensIncentiveAprData?: MarketTokenAPRs;
  currentMarketInfo?: MarketInfo;
};

export default function MarketTokenSelector(props: Props) {
  const { marketsTokensIncentiveAprData, marketsTokensAPRData, marketsInfoData, marketTokensData, currentMarketInfo } =
    props;
  const { markets: sortedMarketsByIndexToken } = useSortedPoolsWithIndexToken(marketsInfoData, marketTokensData);
  const [searchKeyword, setSearchKeyword] = useState("");
  const navigate = useNavigate();
  const indexName = currentMarketInfo && getMarketIndexName(currentMarketInfo);
  const poolName = currentMarketInfo && getMarketPoolName(currentMarketInfo);

  const filteredTokens = useMemo(() => {
    if (sortedMarketsByIndexToken.length < 1) {
      return [];
    }
    if (searchKeyword.length < 1) {
      return sortedMarketsByIndexToken;
    }

    return sortedMarketsByIndexToken.filter((market) => {
      const marketInfo = getByKey(marketsInfoData, market?.address.toBase58())!;
      return marketInfo.name.toLowerCase().indexOf(searchKeyword.toLowerCase()) > -1;
    });
  }, [marketsInfoData, searchKeyword, sortedMarketsByIndexToken]);

  const filteredTokensInfo = useMemo(() => {
    return filteredTokens.map((market) => {
      const marketInfo = getByKey(marketsInfoData, market?.address.toBase58())!;
      // const mintableInfo = getMintableMarketTokens(marketInfo, market);
      const sellableInfo = getSellableMarketToken(marketInfo, market);
      const apr = getByKey(marketsTokensAPRData, market?.address.toBase58());
      const incentiveApr = getByKey(marketsTokensIncentiveAprData, marketInfo?.marketTokenAddress.toBase58());
      const indexName = getMarketIndexName(marketInfo);
      const poolName = getMarketPoolName(marketInfo);
      return {
        market,
        // mintableInfo,
        sellableInfo,
        marketInfo,
        indexName,
        poolName,
        apr,
        incentiveApr,
      };
    });
  }, [filteredTokens, marketsInfoData, marketsTokensAPRData, marketsTokensIncentiveAprData]);

  function handleSelectToken(marketTokenAddress: string) {
    navigate(`/earn?market=${marketTokenAddress}`);
  }

  return (
    <Popover className="MarketTokenSelector">
      {({ open, close }) => {
        // TODO: remove this setstate in render
        if (!open && searchKeyword.length > 0) setSearchKeyword("");
        if (!currentMarketInfo) return <></>;

        const { indexToken, longToken, shortToken } = currentMarketInfo;
        const iconName = currentMarketInfo?.isSpotOnly
          ? getNormalizedTokenSymbol(longToken.symbol) + getNormalizedTokenSymbol(shortToken.symbol)
          : indexToken.symbol;

        return (
          <div>
            <Popover.Button as="div">
              <button className={cx("chart-token-selector")}>
                <span className="chart-token-selector--current inline-items-center">
                  {currentMarketInfo && (
                    <>
                      <TokenIcon
                        className="chart-token-current-icon"
                        symbol={iconName}
                        displaySize={30}
                        importSize={40}
                      />
                      <div className="Market-index-name">
                        <div className="items-center">
                          <span>GM{indexName && `: ${indexName}`}</span>
                          <span className="subtext">{poolName && `[${poolName}]`}</span>
                        </div>
                        <div className="Market-subtext">GMSOL Market Tokens</div>
                      </div>
                    </>
                  )}
                </span>
                <FaChevronDown fontSize={14} />
              </button>
            </Popover.Button>
            <div className="chart-token-menu">
              <Popover.Panel as="div" className="menu-items chart-token-menu-items">
                <SearchInput
                  className="m-md"
                  value={searchKeyword}
                  setValue={({ target }) => setSearchKeyword(target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && filteredTokens.length > 0) {
                      handleSelectToken(filteredTokens[0].address.toBase58());
                      close();
                    }
                  }}
                  placeholder="Search Market"
                />
                <div className="divider" />
                <div className="chart-token-list">
                  <table>
                    {sortedMarketsByIndexToken.length > 0 && (
                      <thead className="table-head">
                        <tr>
                          <th>MARKET</th>
                          <th>BUYABLE</th>
                          <th>SELLABLE</th>
                          <th>APR</th>
                        </tr>
                      </thead>
                    )}
                    <tbody>
                      {filteredTokensInfo.map(
                        ({
                          market,
                          // mintableInfo,
                          sellableInfo,
                          // apr,
                          // incentiveApr,
                          marketInfo,
                          poolName,
                          indexName,
                        }) => {
                          const { indexToken, longToken, shortToken } = marketInfo;
                          const iconName = marketInfo.isSpotOnly
                            ? getNormalizedTokenSymbol(longToken.symbol) + getNormalizedTokenSymbol(shortToken.symbol)
                            : getNormalizedTokenSymbol(indexToken.symbol);
                          return (
                            <Popover.Button
                              as="tr"
                              key={market.address.toBase58()}
                              onClick={() => handleSelectToken(market.address.toBase58())}
                            >
                              <td className="token-item">
                                <span className="inline-items-center">
                                  {marketInfo && (
                                    <>
                                      <TokenIcon
                                        className="ChartToken-list-icon"
                                        symbol={iconName}
                                        displaySize={16}
                                        importSize={40}
                                      />
                                      <div className="items-center">
                                        <span>{indexName && indexName}</span>
                                        <span className="subtext lh-1">{poolName && `[${poolName}]`}</span>
                                      </div>
                                    </>
                                  )}
                                </span>
                              </td>
                              <td>
                                {/* {formatUsd(mintableInfo?.mintableUsd, {
                                  displayDecimals: 0,
                                  fallbackToZero: true,
                                })} */}
                                Unlimited (dev only)
                              </td>
                              <td>
                                {formatTokenAmount(sellableInfo?.totalAmount ?? BN_ZERO, market?.decimals, market?.symbol, {
                                  displayDecimals: 0,
                                  useCommas: true,
                                })}
                              </td>
                              <td>
                                {/* <AprInfo apr={apr} incentiveApr={incentiveApr} showTooltip={false} /> */}
                                Unavailable
                              </td>
                            </Popover.Button>
                          );
                        }
                      )}
                    </tbody>
                  </table>
                </div>
              </Popover.Panel>
            </div>
          </div>
        );
      }}
    </Popover>
  );
}
