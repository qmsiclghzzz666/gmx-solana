import { useSharedStatesSelector } from "@/contexts/shared";
import { selectTradeBoxAvailableTokensOptions, selectTradeBoxFromTokenAddress, selectTradeBoxSetFromTokenAddress, selectTradeBoxState, selectTradeBoxTradeFlags } from "@/contexts/shared/selectors/trade-box-selectors";
import { ChangeEvent, FormEventHandler, useCallback } from "react";
import BuyInputSection from "../BuyInputSection/BuyInputSection";
import { t } from "@lingui/macro";
import TokenSelector from "../TokenSelector/TokenSelector";
import { Token, TokenData, getTokenData } from "@/onchain/token";
import { convertToUsd, formatTokenAmount, parseValue } from "@/utils/number";
import { createSharedStatesSelector } from "@/contexts/shared/utils";
import { selectMarketStateTokens } from "@/contexts/shared/selectors/market-selectors";
import { BN_ZERO } from "@/config/constants";
import { BN } from "@coral-xyz/anchor";
import { formatUsd } from "../MarketsList/utils";

const parseAmount = (value: string, token?: Token) => (token ? parseValue(value || "0", token.decimals) : BN_ZERO) ?? BN_ZERO;
const calcUsd = (amount: BN, token?: TokenData) => convertToUsd(amount, token?.decimals, token?.prices.minPrice);

const selectFromToken = createSharedStatesSelector([selectMarketStateTokens, selectTradeBoxFromTokenAddress], (tokens, address) => getTokenData(tokens, address));
const selectFromTokenInputValue = createSharedStatesSelector([selectTradeBoxState], state => state.fromTokenInputValue);
const selectSetFromTokenInputValue = createSharedStatesSelector([selectTradeBoxState], state => state.setFromTokenInputValue);
const selectFromTokenInputAmount = createSharedStatesSelector([selectFromTokenInputValue, selectFromToken], parseAmount);
const selectFromTokenUsd = createSharedStatesSelector([selectFromTokenInputAmount, selectFromToken], calcUsd);
const selectSwapTokens = createSharedStatesSelector([selectTradeBoxAvailableTokensOptions], options => options.swapTokens);
const selectSortedLongAndShortTokens = createSharedStatesSelector([selectTradeBoxAvailableTokensOptions], options => options.sortedLongAndShortTokens);

export function TradeForm() {
  const { isSwap, isIncrease, isPosition, isLimit, isTrigger } = useSharedStatesSelector(selectTradeBoxTradeFlags);

  const handleSubmit: FormEventHandler<HTMLFormElement> = useCallback((e) => {
    e.preventDefault();
  }, []);

  return (
    <form onSubmit={handleSubmit}>
      {(isSwap || isIncrease) && <TokenInputs />}
      {isTrigger && <DecreaseSizeInput />}
      {isSwap && isLimit && <TriggerRatioInput />}
      {isPosition && (isLimit || isTrigger) && <TriggerPriceInput />}
      <TradeInfo />
    </form>
  );
}

function TokenInputs() {
  const fromToken = useSharedStatesSelector(selectFromToken);
  const fromUsd = useSharedStatesSelector(selectFromTokenUsd);
  const fromTokenInputValue = useSharedStatesSelector(selectFromTokenInputValue);
  const swapTokens = useSharedStatesSelector(selectSwapTokens);
  const tokens = useSharedStatesSelector(selectMarketStateTokens);
  const sortedLongAndShortTokens = useSharedStatesSelector(selectSortedLongAndShortTokens);
  const setFromTokenInputValue = useSharedStatesSelector(selectSetFromTokenInputValue);
  const setFromTokenAddress = useSharedStatesSelector(selectTradeBoxSetFromTokenAddress);

  const isFromTokenInitialized = fromToken?.balance !== null;

  const handleFromInputTokenChange = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    setFromTokenInputValue(e.target.value);
  }, [setFromTokenInputValue]);
  const handleSelectFromToken = useCallback((token: Token) => {
    setFromTokenAddress(token.address.toBase58());
  }, [setFromTokenAddress]);

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
  return (
    <></>
  );
}
