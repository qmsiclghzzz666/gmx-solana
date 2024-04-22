import Tab from "@/components/Tab/Tab";
import { Mode, Operation, getGmSwapBoxAvailableModes } from "./utils";
import { Dispatch, SetStateAction, useCallback, useEffect, useMemo } from "react";
import { useLingui } from "@lingui/react";
import { mapValues } from "lodash";
import { useSafeState } from "@/utils/state";
import { getByKey } from "@/utils/objects";
import { MarketInfos } from "@/onchain/market";
import { PublicKey } from "@solana/web3.js";
import cx from "classnames";
import { t, Trans } from "@lingui/macro";

import "./GmSwapBox.scss";
import { formatUsd } from "@/components/MarketsList/utils";
import { formatTokenAmount } from "@/utils/number";

const OPERATION_LABELS = {
  [Operation.Deposit]: /*i18n*/ "Buy GM",
  [Operation.Withdrawal]: /*i18n*/ "Sell GM",
};

const MODE_LABELS = {
  [Mode.Single]: /*i18n*/ "Single",
  [Mode.Pair]: /*i18n*/ "Pair",
};

export default function Inner({
  operation,
  setOperation,
  mode,
  setMode,
  marketsInfoData,
  marketAddress,
}: {
  operation: Operation
  setOperation: Dispatch<SetStateAction<Operation>>,
  mode: Mode,
  setMode: Dispatch<SetStateAction<Mode>>,
  marketsInfoData?: MarketInfos,
  marketAddress?: string,
}) {
  const { i18n } = useLingui();

  const { localizedOperationLabels, localizedModeLabels } = useMemo(() => {
    return {
      localizedOperationLabels: mapValues(OPERATION_LABELS, (label) => i18n._(label)),
      localizedModeLabels: mapValues(MODE_LABELS, (label) => i18n._(label)),
    };
  }, [i18n]);

  const [firstTokenInputValue, setFirstTokenInputValue] = useSafeState<string>("");
  const [secondTokenInputValue, setSecondTokenInputValue] = useSafeState<string>("");
  const [marketTokenInputValue, setMarketTokenInputValue] = useSafeState<string>("");

  const resetInputs = useCallback(() => {
    setFirstTokenInputValue("");
    setSecondTokenInputValue("");
    setMarketTokenInputValue("");
  }, [setFirstTokenInputValue, setMarketTokenInputValue, setSecondTokenInputValue]);

  const onOperationChange = useCallback(
    (operation: Operation) => {
      resetInputs();
      setOperation(operation);
    },
    [resetInputs, setOperation]
  );

  const marketInfo = getByKey(marketsInfoData, marketAddress);
  const availableModes = useMemo(() => getGmSwapBoxAvailableModes(operation, marketInfo), [operation, marketInfo]);

  const isDeposit = operation === Operation.Deposit;
  const isWithdrawal = operation === Operation.Withdrawal;

  return (
    <div className={`App-box GmSwapBox`}>
      <Tab
        options={Object.values(Operation)}
        optionLabels={localizedOperationLabels}
        option={operation}
        onChange={onOperationChange}
        className="Exchange-swap-option-tabs"
      />

      <Tab
        options={availableModes}
        optionLabels={localizedModeLabels}
        className="GmSwapBox-asset-options-tabs"
        type="inline"
        option={mode}
        onChange={setMode}
      />

      {/* <form
      // onSubmit={(e) => {
      //   e.preventDefault();
      //   submitState.onSubmit();
      // }}
      >
        <div className={cx("GmSwapBox-form-layout", { reverse: isWithdrawal })}>
          <BuyInputSection
            topLeftLabel={isDeposit ? t`Pay` : t`Receive`}
            topLeftValue={formatUsd(firstTokenUsd)}
            topRightLabel={t`Balance`}
            topRightValue={formatTokenAmount(firstToken?.balance, firstToken?.decimals, "", {
              useCommas: true,
            })}
            preventFocusOnLabelClick="right"
            {...(isDeposit && {
              onClickTopRightLabel: onMaxClickFirstToken,
            })}
            showMaxButton={
              isDeposit &&
              firstToken?.balance?.gt(0) &&
              !firstTokenAmount?.eq(firstToken.balance) &&
              (firstToken?.isNative ? minResidualAmount && firstToken?.balance?.gt(minResidualAmount) : true)
            }
            inputValue={firstTokenInputValue}
            onInputValueChange={(e) => {
              if (firstToken) {
                setFirstTokenInputValue(e.target.value);
                onFocusedCollateralInputChange(firstToken.address);
              }
            }}
            onClickMax={onMaxClickFirstToken}
          >
            {firstTokenAddress && isSingle ? (
              <TokenSelector
                label={isDeposit ? t`Pay` : t`Receive`}
                chainId={chainId}
                tokenAddress={firstTokenAddress}
                onSelectToken={(token) => setFirstTokenAddress(token.address)}
                tokens={tokenOptions}
                infoTokens={infoTokens}
                className="GlpSwap-from-token"
                showSymbolImage={true}
                showTokenImgInDropdown={true}
              />
            ) : (
              <div className="selected-token">
                <TokenWithIcon symbol={firstToken?.symbol} displaySize={20} />
              </div>
            )}
          </BuyInputSection>

          {isPair && secondTokenAddress && (
            <BuyInputSection
              topLeftLabel={isDeposit ? t`Pay` : t`Receive`}
              topLeftValue={formatUsd(secondTokenUsd)}
              topRightLabel={t`Balance`}
              topRightValue={formatTokenAmount(secondToken?.balance, secondToken?.decimals, "", {
                useCommas: true,
              })}
              preventFocusOnLabelClick="right"
              inputValue={secondTokenInputValue}
              showMaxButton={
                isDeposit &&
                secondToken?.balance?.gt(0) &&
                !secondTokenAmount?.eq(secondToken.balance) &&
                (secondToken?.isNative ? minResidualAmount && secondToken?.balance?.gt(minResidualAmount) : true)
              }
              onInputValueChange={(e) => {
                if (secondToken) {
                  setSecondTokenInputValue(e.target.value);
                  onFocusedCollateralInputChange(secondToken.address);
                }
              }}
              {...(isDeposit && {
                onClickTopRightLabel: onMaxClickSecondToken,
              })}
              onClickMax={onMaxClickSecondToken}
            >
              <div className="selected-token">
                <TokenWithIcon symbol={secondToken?.symbol} displaySize={20} />
              </div>
            </BuyInputSection>
          )}

          <div className="AppOrder-ball-container" onClick={onSwitchSide}>
            <div className="AppOrder-ball">
              <IoMdSwap className="Exchange-swap-ball-icon" />
            </div>
          </div>

          <BuyInputSection
            topLeftLabel={isWithdrawal ? t`Pay` : t`Receive`}
            topLeftValue={marketTokenUsd?.gt(0) ? formatUsd(marketTokenUsd) : ""}
            topRightLabel={t`Balance`}
            topRightValue={formatTokenAmount(marketToken?.balance, marketToken?.decimals, "", {
              useCommas: true,
            })}
            preventFocusOnLabelClick="right"
            showMaxButton={isWithdrawal && marketToken?.balance?.gt(0) && !marketTokenAmount?.eq(marketToken.balance)}
            inputValue={marketTokenInputValue}
            onInputValueChange={(e) => {
              setMarketTokenInputValue(e.target.value);
              setFocusedInput("market");
            }}
            {...(isWithdrawal && {
              onClickTopRightLabel: () => {
                if (marketToken?.balance) {
                  setMarketTokenInputValue(formatAmountFree(marketToken.balance, marketToken.decimals));
                  setFocusedInput("market");
                }
              },
            })}
            onClickMax={() => {
              if (marketToken?.balance) {
                const formattedGMBalance = formatAmountFree(marketToken.balance, marketToken.decimals);
                const finalGMBalance = isMetamaskMobile
                  ? limitDecimals(formattedGMBalance, MAX_METAMASK_MOBILE_DECIMALS)
                  : formattedGMBalance;
                setMarketTokenInputValue(finalGMBalance);
                setFocusedInput("market");
              }
            }}
          >
            <PoolSelector
              label={t`Pool`}
              className="SwapBox-info-dropdown"
              selectedIndexName={indexName}
              selectedMarketAddress={marketAddress}
              markets={sortedMarketsInfoByIndexToken}
              marketTokensData={marketTokensData}
              isSideMenu
              showBalances
              showAllPools
              showIndexIcon
              onSelectMarket={(marketInfo) => {
                setIndexName(getMarketIndexName(marketInfo));
                onMarketChange(marketInfo.marketTokenAddress);
                showMarketToast(marketInfo);
              }}
            />
          </BuyInputSection>
        </div>

        <ExchangeInfo className="GmSwapBox-info-section" dividerClassName="App-card-divider">
          <ExchangeInfo.Group>
            <ExchangeInfoRow
              className="SwapBox-info-row"
              label={t`Pool`}
              value={
                <PoolSelector
                  label={t`Pool`}
                  className="SwapBox-info-dropdown"
                  selectedIndexName={indexName}
                  selectedMarketAddress={marketAddress}
                  markets={markets}
                  marketTokensData={marketTokensData}
                  isSideMenu
                  showBalances
                  onSelectMarket={(marketInfo) => {
                    onMarketChange(marketInfo.marketTokenAddress);
                    showMarketToast(marketInfo);
                  }}
                />
              }
            />
          </ExchangeInfo.Group>

          <ExchangeInfo.Group>
            <div className="GmSwapBox-info-section">
              <GmFees
                isDeposit={isDeposit}
                totalFees={fees?.totalFees}
                swapFee={fees?.swapFee}
                swapPriceImpact={fees?.swapPriceImpact}
                uiFee={fees?.uiFee}
              />
              <NetworkFeeRow executionFee={executionFee} />
            </div>
          </ExchangeInfo.Group>

          {isHighPriceImpact && (
            <ExchangeInfo.Group>
              <Checkbox
                className="GmSwapBox-warning"
                asRow
                isChecked={isHighPriceImpactAccepted}
                setIsChecked={setIsHighPriceImpactAccepted}
              >
                {isSingle ? (
                  <Tooltip
                    className="warning-tooltip"
                    handle={<Trans>Acknowledge high Price Impact</Trans>}
                    position="top-start"
                    renderContent={() => (
                      <div>{t`Consider selecting and using the "Pair" option to reduce the Price Impact.`}</div>
                    )}
                  />
                ) : (
                  <span className="muted font-sm text-warning">
                    <Trans>Acknowledge high Price Impact</Trans>
                  </span>
                )}
              </Checkbox>
            </ExchangeInfo.Group>
          )}
        </ExchangeInfo>

        <div className="Exchange-swap-button-container">
          <Button
            className="w-full"
            variant="primary-action"
            onClick={submitState.onSubmit}
            disabled={submitState.isDisabled}
          >
            {submitState.text}
          </Button>
        </div>
      </form> */}
      {/* 
      <GmConfirmationBox
        isVisible={stage === "confirmation"}
        marketToken={marketToken!}
        longToken={longTokenInputState?.token}
        shortToken={shortTokenInputState?.token}
        marketTokenAmount={amounts?.marketTokenAmount ?? BigNumber.from(0)}
        marketTokenUsd={amounts?.marketTokenUsd ?? BigNumber.from(0)}
        longTokenAmount={amounts?.longTokenAmount}
        longTokenUsd={amounts?.longTokenUsd}
        shortTokenAmount={amounts?.shortTokenAmount}
        shortTokenUsd={amounts?.shortTokenUsd}
        fees={fees!}
        error={submitState.error}
        isDeposit={isDeposit}
        executionFee={executionFee}
        onSubmitted={() => {
          setStage("swap");
        }}
        onClose={() => {
          setStage("swap");
        }}
        shouldDisableValidation={shouldDisableValidationForTesting}
      /> */}
    </div>
  );
}
