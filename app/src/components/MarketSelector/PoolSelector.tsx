import { t } from "@lingui/macro";
import cx from "classnames";
import { KeyboardEventHandler, useMemo, useState } from "react";
import { BiChevronDown } from "react-icons/bi";
import Modal from "../Modal/Modal";
import TooltipWithPortal from "../Tooltip/TooltipWithPortal";
import "./MarketSelector.scss";
import { MarketInfo } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { BN } from "@coral-xyz/anchor";
import { formatUsd, getMarketIndexName, getMarketPoolName } from "../MarketsList/utils";
import { getByKey } from "@/utils/objects";
import { convertToUsd, formatTokenAmount } from "@/utils/number";
import { BN_ZERO } from "@/config/constants";
import SearchInput from "../SearchInput/SearchInput";
import { getNormalizedTokenSymbol } from "@/utils/tokens";
import TokenIcon from "../TokenIcon/TokenIcon";

type Props = {
  label?: string;
  className?: string;
  selectedMarketAddress?: string;
  selectedIndexName?: string;
  markets: MarketInfo[];
  marketTokensData?: Tokens;
  showBalances?: boolean;
  isSideMenu?: boolean;
  getMarketState?: (market: MarketInfo) => MarketState | undefined;
  onSelectMarket: (market: MarketInfo) => void;
  showAllPools?: boolean;
  showIndexIcon?: boolean;
};

type MarketState = {
  disabled?: boolean;
  message?: string;
};

type MarketOption = {
  indexName: string;
  poolName: string;
  name: string;
  marketInfo: MarketInfo;
  balance: BN;
  balanceUsd: BN;
  state?: MarketState;
};

export function PoolSelector({
  selectedMarketAddress,
  className,
  selectedIndexName,
  label,
  markets,
  isSideMenu,
  marketTokensData,
  showBalances,
  onSelectMarket,
  getMarketState,
  showAllPools = false,
  showIndexIcon = false,
}: Props) {
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [searchKeyword, setSearchKeyword] = useState("");

  const marketsOptions: MarketOption[] = useMemo(() => {
    const allMarkets = markets
      .filter((market) => !market.isDisabled && (showAllPools || getMarketIndexName(market) === selectedIndexName))
      .map((marketInfo) => {
        const indexName = getMarketIndexName(marketInfo);
        const poolName = getMarketPoolName(marketInfo);
        const marketToken = getByKey(marketTokensData, marketInfo.marketTokenAddress.toBase58());
        const gmBalance = marketToken?.balance;
        const gmBalanceUsd = convertToUsd(marketToken?.balance ?? BN_ZERO, marketToken?.decimals, marketToken?.prices.minPrice);
        const state = getMarketState?.(marketInfo);

        return {
          indexName,
          poolName,
          name: marketInfo.name,
          marketInfo,
          balance: gmBalance || BN_ZERO,
          balanceUsd: gmBalanceUsd || BN_ZERO,
          state,
        };
      });
    const marketsWithBalance: MarketOption[] = [];
    const marketsWithoutBalance: MarketOption[] = [];

    for (const market of allMarkets) {
      if (market.balance.gt(BN_ZERO)) {
        marketsWithBalance.push(market);
      } else {
        marketsWithoutBalance.push(market);
      }
    }

    const sortedMartketsWithBalance = marketsWithBalance.sort((a, b) => {
      return b.balanceUsd?.gt(a.balanceUsd || 0) ? 1 : -1;
    });

    return [...sortedMartketsWithBalance, ...marketsWithoutBalance];
  }, [getMarketState, marketTokensData, markets, selectedIndexName, showAllPools]);

  const marketInfo = useMemo(
    () => marketsOptions.find((option) => option.marketInfo.marketTokenAddress.toBase58() === selectedMarketAddress)?.marketInfo,
    [marketsOptions, selectedMarketAddress]
  );

  const filteredOptions = useMemo(() => {
    const lowercaseSearchKeyword = searchKeyword.toLowerCase();
    return marketsOptions.filter((option) => {
      const name = option.name.toLowerCase();
      return name.includes(lowercaseSearchKeyword);
    });
  }, [marketsOptions, searchKeyword]);

  function onSelectOption(option: MarketOption) {
    onSelectMarket(option.marketInfo);
    setIsModalVisible(false);
  }

  const _handleKeyDown: KeyboardEventHandler<HTMLInputElement> = (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      if (filteredOptions.length > 0) {
        onSelectOption(filteredOptions[0]);
      }
    }
  };

  function displayPoolLabel(marketInfo: MarketInfo | undefined) {
    if (!marketInfo) return "...";
    const name = showAllPools ? `GM: ${getMarketIndexName(marketInfo)}` : getMarketPoolName(marketInfo);

    if (marketsOptions?.length > 1) {
      return (
        <div className="TokenSelector-box" onClick={() => setIsModalVisible(true)}>
          {name ? name : "..."}
          <BiChevronDown className="TokenSelector-caret" />
        </div>
      );
    }

    return <div>{name ? name : "..."}</div>;
  }

  return (
    <div className={cx("TokenSelector", "MarketSelector", { "side-menu": isSideMenu }, className)}>
      <Modal
        isVisible={isModalVisible}
        setIsVisible={setIsModalVisible}
        label={label}
        headerContent={() => (
          <SearchInput
            className="mt-md"
            value={searchKeyword}
            setValue={(e) => setSearchKeyword(e.target.value)}
            placeholder={t`Search Pool`}
            onKeyDown={_handleKeyDown}
          />
        )}
      >
        <div className="TokenSelector-tokens">
          {filteredOptions.map((option, marketIndex) => {
            const { marketInfo, balance, balanceUsd, indexName, poolName, name, state = {} } = option;
            const { longToken, shortToken, indexToken } = marketInfo;

            const indexTokenImage = marketInfo.isSpotOnly
              ? getNormalizedTokenSymbol(longToken.symbol) + getNormalizedTokenSymbol(shortToken.symbol)
              : getNormalizedTokenSymbol(indexToken.symbol);

            const marketToken = getByKey(marketTokensData, marketInfo.marketTokenAddress.toBase58());

            return (
              <div
                key={name}
                className={cx("TokenSelector-token-row", { disabled: state.disabled })}
                onClick={() => !state.disabled && onSelectOption(option)}
              >
                {state.disabled && state.message && (
                  <TooltipWithPortal
                    className="TokenSelector-tooltip"
                    handle={<div className="TokenSelector-tooltip-backing" />}
                    position={marketIndex < filteredOptions.length / 2 ? "bottom" : "top"}
                    disableHandleStyle
                    closeOnDoubleClick
                    fitHandleWidth
                    renderContent={() => state.message}
                  />
                )}
                <div className="Token-info">
                  <div className="collaterals-logo">
                    {showAllPools ? (
                      <TokenIcon symbol={indexTokenImage} displaySize={40} importSize={40} />
                    ) : (
                      <>
                        <TokenIcon
                          symbol={longToken.symbol}
                          displaySize={40}
                          importSize={40}
                          className="collateral-logo collateral-logo-first"
                        />
                        {shortToken && (
                          <TokenIcon
                            symbol={shortToken.symbol}
                            displaySize={40}
                            importSize={40}
                            className="collateral-logo collateral-logo-second"
                          />
                        )}
                      </>
                    )}
                  </div>
                  <div className="Token-symbol">
                    <div className="Token-text">
                      {showAllPools ? (
                        <div className="lh-1 items-center">
                          <span>{indexName && indexName}</span>
                          <span className="subtext">{poolName && `[${poolName}]`}</span>
                        </div>
                      ) : (
                        <div className="Token-text">{poolName}</div>
                      )}
                    </div>
                  </div>
                </div>
                <div className="Token-balance">
                  {showBalances && balance && (
                    <div className="Token-text">
                      {balance.gt(BN_ZERO) &&
                        formatTokenAmount(balance, marketToken?.decimals, "GM", {
                          useCommas: true,
                        })}
                      {balance.eq(BN_ZERO) && "-"}
                    </div>
                  )}
                  <span className="text-accent">
                    {showBalances && balanceUsd && balanceUsd.gt(BN_ZERO) && <div>{formatUsd(balanceUsd)}</div>}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </Modal>

      {marketInfo && (
        <div className="inline-items-center">
          {showIndexIcon && (
            <TokenIcon
              className="mr-xs"
              symbol={
                marketInfo.isSpotOnly
                  ? getNormalizedTokenSymbol(marketInfo.longToken.symbol) +
                  getNormalizedTokenSymbol(marketInfo.shortToken.symbol)
                  : marketInfo?.indexToken.symbol
              }
              importSize={40}
              displaySize={20}
            />
          )}
          {displayPoolLabel(marketInfo)}
        </div>
      )}
    </div>
  );
}
