import { useSharedStatesSelector } from "@/contexts/shared";
import { selectIncreaseAmounts, selectMarketInfo, selectSetMarketAddress, selectTradeBoxChooseSuitableMarket, selectTradeBoxSetFromTokenAddress, selectTradeBoxTradeFlags, selectTradeBoxTradeType } from "@/contexts/shared/selectors/trade-box-selectors";
import { ChangeEvent, FormEventHandler, useCallback, useMemo } from "react";
import BuyInputSection from "../BuyInputSection/BuyInputSection";
import { t } from "@lingui/macro";
import TokenSelector from "../TokenSelector/TokenSelector";
import { Token } from "@/onchain/token";
import { formatTokenAmount } from "@/utils/number";
import { selectMarketStateTokens } from "@/contexts/shared/selectors/market-selectors";
import { BN_ZERO } from "@/config/constants";
import { formatUsd, getMarketIndexName } from "../MarketsList/utils";
import { IoMdSwap } from "react-icons/io";

import { selectFromToken, selectFromTokenInputValue, selectFromTokenUsd, selectSetFromTokenInputValue, selectSetToTokenInputValue, selectSortedAllMarkets, selectSortedLongAndShortTokens, selectSwapTokens, selectSwitchTokenAddresses, selectToToken, selectToTokenInputValue } from "@/contexts/shared/selectors/trade-box-selectors";
import TokenIcon from "../TokenIcon/TokenIcon";
import { TradeType } from "@/onchain/trade";
import { MarketSelector } from "../MarketSelector/MarketSelector";
import Button from "../Button/Button";
import { useSetTradeStage } from "@/contexts/shared/hooks/use-set-trade-stage";
import { ExchangeInfo } from "../Exchange/ExchangeInfo";
import { MarketPoolSelectorRow } from "./MarketPoolSelectorRow";
import { CollateralSelectorRow } from "./CollateralSelectorRow";

const tradeTypeLabels = {
  [TradeType.Long]: t`Long`,
  [TradeType.Short]: t`Short`,
  [TradeType.Swap]: t`Swap`,
};

export function TradeForm() {
  const { isSwap, isIncrease, isPosition, isLimit, isTrigger, isLong } = useSharedStatesSelector(selectTradeBoxTradeFlags);
  const setTradeStage = useSetTradeStage();
  const handleSubmit: FormEventHandler<HTMLFormElement> = useCallback((e) => {
    e.preventDefault();
    setTradeStage("confirmation");
  }, [setTradeStage]);

  const buttonText = useMemo(() => {
    if (isSwap) {
      return t`Swap`
    } else if (isLong) {
      return t`Open Long`
    } else {
      return t`Open Short`
    }
  }, [isLong, isSwap]);

  return (
    <form onSubmit={handleSubmit}>
      {(isSwap || isIncrease) && <TokenInputs isSwap={isSwap} isIncrease={isIncrease} />}
      {isTrigger && <DecreaseSizeInput />}
      {isSwap && isLimit && <TriggerRatioInput />}
      {isPosition && (isLimit || isTrigger) && <TriggerPriceInput />}
      <TradeInfo />
      <div className="Exchange-swap-button-container">
        <Button
          variant="primary-action"
          className="w-full"
          // onClick={onSubmit}
          type="submit"
        // disabled={isSubmitButtonDisabled && !shouldDisableValidationForTesting}
        >
          {buttonText}
        </Button>
      </div>
    </form>
  );
}

function TokenInputs({ isSwap, isIncrease }: { isSwap: boolean, isIncrease: boolean }) {
  const tradeType = useSharedStatesSelector(selectTradeBoxTradeType);
  const fromToken = useSharedStatesSelector(selectFromToken);
  const toToken = useSharedStatesSelector(selectToToken);
  const fromUsd = useSharedStatesSelector(selectFromTokenUsd);
  const fromTokenInputValue = useSharedStatesSelector(selectFromTokenInputValue);
  const toTokenInputValue = useSharedStatesSelector(selectToTokenInputValue);
  const swapTokens = useSharedStatesSelector(selectSwapTokens);
  const tokens = useSharedStatesSelector(selectMarketStateTokens);
  const sortedLongAndShortTokens = useSharedStatesSelector(selectSortedLongAndShortTokens);
  const sortedAllMarkets = useSharedStatesSelector(selectSortedAllMarkets);
  const setFromTokenInputValueRaw = useSharedStatesSelector(selectSetFromTokenInputValue);
  const setToTokenInputValueRaw = useSharedStatesSelector(selectSetToTokenInputValue);
  const setFromTokenAddress = useSharedStatesSelector(selectTradeBoxSetFromTokenAddress);
  const switchTokenAddresses = useSharedStatesSelector(selectSwitchTokenAddresses);
  const chooseSuitableMarket = useSharedStatesSelector(selectTradeBoxChooseSuitableMarket);
  const increaseAmounts = useSharedStatesSelector(selectIncreaseAmounts);

  const setFromTokenInputValue = useCallback((value: string, shouldResetPriceImpactWarning: boolean) => {
    setFromTokenInputValueRaw(value);
    if (shouldResetPriceImpactWarning) {
      // setIsHighPositionImpactAcceptedRef.current(false);
      // setIsHighSwapImpactAcceptedRef.current(false);
    }
  }, [setFromTokenInputValueRaw]);

  const setToTokenInputValue = useCallback((value: string, shouldResetPriceImpactWarning: boolean) => {
    setToTokenInputValueRaw(value);
    if (shouldResetPriceImpactWarning) {
      // setIsHighPositionImpactAcceptedRef.current(false);
      // setIsHighSwapImpactAcceptedRef.current(false);
    }
  }, [setToTokenInputValueRaw]);

  const handleFromInputTokenChange = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    setFromTokenInputValue(e.target.value, true);
  }, [setFromTokenInputValue]);

  const handleToInputTokenChange = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    setToTokenInputValue(e.target.value, true);
  }, [setToTokenInputValue]);

  const handleSelectFromToken = useCallback((token: Token) => {
    setFromTokenAddress(token.address.toBase58());
  }, [setFromTokenAddress]);

  const handleSelectToToken = useCallback((token: Token) => {
    chooseSuitableMarket(token.address.toBase58());
  }, [chooseSuitableMarket]);

  const hanldeSwitchTokens = useCallback(() => {
    switchTokenAddresses();
    setFromTokenInputValue(toTokenInputValue, true);
    setToTokenInputValue(fromTokenInputValue, true);
  }, [fromTokenInputValue, setFromTokenInputValue, setToTokenInputValue, switchTokenAddresses, toTokenInputValue]);

  const isFromTokenInitialized = fromToken?.balance !== null;
  const isToTokenInitialized = toToken?.balance !== null;

  return (
    <>
      <BuyInputSection
        topLeftLabel={t`Pay`}
        // FIXME: the comparison seems to be trivial.
        // Original version is `topLeftValue={fromUsd?.gt(BN_ZERO) ? formatUsd(isIncrease ? increaseAmounts?.initialCollateralUsd : fromUsd) : ""}`
        topLeftValue={fromUsd?.gt(BN_ZERO) ? formatUsd(fromUsd) : ""}
        topRightLabel={isFromTokenInitialized ? t`Balance` : t`Unintialized`}
        topRightValue={isFromTokenInitialized ? formatTokenAmount(fromToken?.balance ?? BN_ZERO, fromToken?.decimals, "", {
          useCommas: true,
        }) : ""}
        // onClickTopRightLabel={onMaxClick}
        inputValue={fromTokenInputValue}
        onInputValueChange={handleFromInputTokenChange}
      // showMaxButton={isNotMatchAvailableBalance}
      // onClickMax={onMaxClick}
      >
        {fromToken && (
          <TokenSelector
            label={t`Pay`}
            // chainId={chainId}
            token={fromToken}
            onSelectToken={handleSelectFromToken}
            tokens={swapTokens}
            infoTokens={tokens}
            className="GlpSwap-from-token"
            showSymbolImage={true}
            showTokenImgInDropdown={true}
            extendedSortSequence={sortedLongAndShortTokens}
          />
        )}
      </BuyInputSection>

      <div className="Exchange-swap-ball-container">
        <button type="button" className="Exchange-swap-ball" onClick={hanldeSwitchTokens}>
          <IoMdSwap className="Exchange-swap-ball-icon" />
        </button>
      </div>

      {isSwap && (
        <BuyInputSection
          topLeftLabel={t`Receive`}
          // topLeftValue={swapAmounts?.usdOut.gt(0) ? formatUsd(swapAmounts?.usdOut) : ""}
          topRightLabel={isToTokenInitialized ? t`Balance` : t`Uninitialized`}
          topRightValue={isToTokenInitialized ? formatTokenAmount(toToken?.balance ?? BN_ZERO, toToken?.decimals, "", {
            useCommas: true,
          }) : ""}
          inputValue={toTokenInputValue}
          onInputValueChange={handleToInputTokenChange}
          showMaxButton={false}
          preventFocusOnLabelClick="right"
        >
          {toToken && (
            <TokenSelector
              label={t`Receive`}
              token={toToken}
              onSelectToken={handleSelectToToken}
              tokens={swapTokens}
              infoTokens={tokens}
              className="GlpSwap-from-token"
              showSymbolImage={true}
              showBalances={true}
              showTokenImgInDropdown={true}
              extendedSortSequence={sortedLongAndShortTokens}
            />
          )}
        </BuyInputSection>
      )}

      {isIncrease && (
        <BuyInputSection
          topLeftLabel={tradeTypeLabels[tradeType]}
          topLeftValue={
            increaseAmounts?.sizeDeltaUsd.gt(BN_ZERO)
              ? formatUsd(increaseAmounts?.sizeDeltaUsd, { fallbackToZero: true })
              : ""
          }
          topRightLabel={t`Leverage`}
          // topRightValue={formatLeverage(isLeverageEnabled ? leverage : increaseAmounts?.estimatedLeverage) || "-"}
          inputValue={toTokenInputValue}
          onInputValueChange={handleToInputTokenChange}
          showMaxButton={false}
        >
          {toToken && (
            <MarketSelector
              label={tradeTypeLabels[tradeType]}
              selectedIndexName={toToken ? getMarketIndexName({ indexToken: toToken, isSpotOnly: false }) : undefined}
              selectedMarketLabel={
                toToken && (
                  <>
                    <span className="inline-items-center">
                      <TokenIcon className="mr-xs" symbol={toToken.symbol} importSize={24} displaySize={20} />
                      <span className="Token-symbol-text">{toToken.symbol}</span>
                    </span>
                  </>
                )
              }
              markets={sortedAllMarkets ?? []}
              isSideMenu
              onSelectMarket={(_indexName, marketInfo) => handleSelectToToken(marketInfo.indexToken)}
            />
          )}
        </BuyInputSection>
      )}
    </>
  );
}

function DecreaseSizeInput() {
  return (
    <></>
  );
}

function TriggerRatioInput() {
  return (
    <></>
  );
}

function TriggerPriceInput() {
  return (
    <></>
  );
}

function TradeInfo() {
  const { isPosition } = useSharedStatesSelector(selectTradeBoxTradeFlags);
  const marketInfo = useSharedStatesSelector(selectMarketInfo);
  const toToken = useSharedStatesSelector(selectToToken);
  const setMarketAddress = useSharedStatesSelector(selectSetMarketAddress);
  function renderPositionControls() {
    return (
      <>
        <MarketPoolSelectorRow
          selectedMarket={marketInfo}
          indexToken={toToken}
          // isOutPositionLiquidity={isOutPositionLiquidity}
          // currentPriceImpactBps={increaseAmounts?.acceptablePriceDeltaBps}
          onSelectMarketAddress={setMarketAddress}
        />

        <CollateralSelectorRow />
      </>
    );
  }
  return (
    <ExchangeInfo className="SwapBox-info-section" dividerClassName="App-card-divider">
      <ExchangeInfo.Group>{isPosition && renderPositionControls()}</ExchangeInfo.Group>
    </ExchangeInfo>
  );
}
