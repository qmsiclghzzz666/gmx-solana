import { t } from "@lingui/macro";
import cx from "classnames";
import { KeyboardEventHandler, ReactNode, useMemo, useState } from "react";
import { BiChevronDown } from "react-icons/bi";
import Modal from "../Modal/Modal";
import TooltipWithPortal from "../Tooltip/TooltipWithPortal";
import "./MarketSelector.scss";
import { MarketInfo } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { BN } from "@coral-xyz/anchor";
import { formatUsd, getMarketIndexName } from "../MarketsList/utils";
import { getByKey } from "@/utils/objects";
import { convertToUsd, formatTokenAmount } from "@/utils/number";
import { BN_ZERO } from "@/config/constants";
import SearchInput from "../SearchInput/SearchInput";
import { getIconUrlPath } from "@/utils/icon";

type Props = {
  label?: string;
  className?: string;
  selectedIndexName?: string;
  markets: MarketInfo[];
  marketTokensData?: Tokens;
  showBalances?: boolean;
  selectedMarketLabel?: ReactNode | string;
  isSideMenu?: boolean;
  getMarketState?: (market: MarketInfo) => MarketState | undefined;
  onSelectMarket: (indexName: string, market: MarketInfo) => void;
};

type MarketState = {
  disabled?: boolean;
  message?: string;
};

type MarketOption = {
  indexName: string;
  marketInfo: MarketInfo;
  balance: BN;
  balanceUsd: BN;
  state?: MarketState;
};

export function MarketSelector({
  selectedIndexName,
  className,
  selectedMarketLabel,
  label,
  markets,
  isSideMenu,
  marketTokensData,
  showBalances,
  onSelectMarket,
  getMarketState,
}: Props) {
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [searchKeyword, setSearchKeyword] = useState("");

  const marketsOptions: MarketOption[] = useMemo(() => {
    const optionsByIndexName: { [indexName: string]: MarketOption } = {};

    markets
      .filter((market) => !market.isDisabled)
      .forEach((marketInfo) => {
        const indexName = getMarketIndexName(marketInfo);
        const marketToken = getByKey(marketTokensData, marketInfo.marketTokenAddress.toBase58());

        const gmBalance = marketToken?.balance;
        const gmBalanceUsd = convertToUsd(marketToken?.balance ?? BN_ZERO, marketToken?.decimals, marketToken?.prices.minPrice);
        const state = getMarketState?.(marketInfo);

        const option = optionsByIndexName[indexName];

        if (option) {
          option.balance = option.balance.add(gmBalance || BN_ZERO);
          option.balanceUsd = option.balanceUsd.add(gmBalanceUsd || BN_ZERO);
        }

        optionsByIndexName[indexName] = optionsByIndexName[indexName] || {
          indexName,
          marketInfo,
          balance: gmBalance || BN_ZERO,
          balanceUsd: gmBalanceUsd || BN_ZERO,
          state,
        };
      });

    return Object.values(optionsByIndexName);
  }, [getMarketState, marketTokensData, markets]);

  const marketInfo = marketsOptions.find((option) => option.indexName === selectedIndexName)?.marketInfo;

  const filteredOptions = marketsOptions.filter((option) => {
    return (
      option.indexName.toLowerCase().indexOf(searchKeyword.toLowerCase()) > -1 ||
      (!option.marketInfo.isSpotOnly &&
        option.marketInfo.indexToken.symbol.toLowerCase().indexOf(searchKeyword.toLowerCase()) > -1)
    );
  });

  function onSelectOption(option: MarketOption) {
    onSelectMarket(option.indexName, option.marketInfo);
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

  return (
    <div className={cx("TokenSelector", "MarketSelector", { "side-menu": isSideMenu }, className)}>
      <Modal
        isVisible={isModalVisible}
        onClose={setIsModalVisible}
        label={label}
        headerContent={() => (
          <SearchInput
            className="mt-md"
            value={searchKeyword}
            setValue={(e) => setSearchKeyword(e.target.value)}
            placeholder={t`Search Market`}
            onKeyDown={_handleKeyDown}
          />
        )}
      >
        <div className="TokenSelector-tokens">
          {filteredOptions.map((option, marketIndex) => {
            const { marketInfo, balance, balanceUsd, indexName, state = {} } = option;
            const assetImage = getIconUrlPath(`${marketInfo.isSpotOnly ? "swap" : marketInfo.indexToken.symbol.toLowerCase()}`, 40);

            const marketToken = getByKey(marketTokensData, marketInfo.marketTokenAddress.toBase58());

            return (
              <div
                key={indexName}
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
                  <img src={assetImage} alt={indexName} className="token-logo" />
                  <div className="Token-symbol">
                    <div className="Token-text">{indexName}</div>
                  </div>
                </div>
                <div className="Token-balance">
                  {showBalances && balance && (
                    <div className="Token-text">
                      {balance.gt(BN_ZERO) &&
                        formatTokenAmount(balance, marketToken?.decimals, "", {
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
      {selectedMarketLabel ? (
        <div className="TokenSelector-box" onClick={() => setIsModalVisible(true)}>
          {selectedMarketLabel}
          <BiChevronDown className="TokenSelector-caret" />
        </div>
      ) : (
        <div className="TokenSelector-box" onClick={() => setIsModalVisible(true)}>
          {marketInfo ? getMarketIndexName(marketInfo) : "..."}
          <BiChevronDown className="TokenSelector-caret" />
        </div>
      )}
    </div>
  );
}
