import Tab from "@/components/Tab/Tab";
import { TokenOptions, getGmSwapBoxAvailableModes } from "../utils";
import { CreateDepositParams, CreateWithdrawalParams, Mode, Operation } from "../types";
import { useCallback, useMemo } from "react";
import { useLingui } from "@lingui/react";
import { mapValues } from "lodash";
import cx from "classnames";
import { t } from "@lingui/macro";

import "./GmSwapBox.scss";
import { formatUsd, getMarketIndexName } from "@/components/MarketsList/utils";
import Button from "@/components/Button/Button";
import BuyInputSection from "@/components/BuyInputSection/BuyInputSection";
import { Token } from "@/onchain/token";
import TokenWithIcon from "@/components/TokenIcon/TokenWithIcon";
import TokenSelector from "@/components/TokenSelector/TokenSelector";
import { useLocalStorageSerializeKey } from "@/utils/localStorage";
import { getSyntheticsDepositIndexTokenKey } from "@/config/localStorage";
import { BN_ZERO } from "@/config/constants";
import { IoMdSwap } from "react-icons/io";
import { PoolSelector } from "@/components/MarketSelector/PoolSelector";
import { useGmInputDisplay, useGmStateDispath, useGmStateSelector, useHandleSumit } from "../hooks";

const OPERATION_LABELS = {
  [Operation.Deposit]: /*i18n*/ "Buy GM",
  [Operation.Withdrawal]: /*i18n*/ "Sell GM",
};

const MODE_LABELS = {
  [Mode.Single]: /*i18n*/ "Single",
  [Mode.Pair]: /*i18n*/ "Pair",
};

export function GmForm({
  genesisHash,
  tokenOptions: { tokenOptions, firstToken, secondToken },
  setOperation,
  setMode,
  onSelectMarket,
  onSelectFirstToken,
  onCreateDeposit,
  onCreateWithdrawal,
}: {
  genesisHash: string,
  tokenOptions: TokenOptions,
  setOperation: (operation: Operation) => void,
  setMode: (mode: Mode) => void,
  onSelectMarket: (marketAddress: string) => void,
  onSelectFirstToken: (token: Token) => void,
  onCreateDeposit: (params: CreateDepositParams) => void,
  onCreateWithdrawal: (params: CreateWithdrawalParams) => void,
}) {
  const { i18n } = useLingui();

  const { localizedOperationLabels, localizedModeLabels } = useMemo(() => {
    return {
      localizedOperationLabels: mapValues(OPERATION_LABELS, (label) => i18n._(label)),
      localizedModeLabels: mapValues(MODE_LABELS, (label) => i18n._(label)),
    };
  }, [i18n]);

  const dispatch = useGmStateDispath();

  const {
    inputState,
    marketInfo,
    marketTokens,
    sortedMarketsInfoByIndexToken
  } = useGmStateSelector(s => {
    return {
      inputState: s.input,
      marketInfo: s.market,
      marketTokens: s.marketTokens,
      sortedMarketsInfoByIndexToken: s.sortedMarketsInfoByIndexToken,
    }
  });
  const { firstTokenUsd, secondTokenUsd, marketTokenUsd } = useGmInputDisplay();
  const { operation, mode } = useGmStateSelector(s => {
    return {
      operation: s.operation,
      mode: s.mode,
    };
  });

  const handleSubmit = useHandleSumit({ onCreateDeposit, onCreateWithdrawal });

  // const [focusedInput, setFocusedInput] = useState<"longCollateral" | "shortCollateral" | "market">("market");

  const resetInputs = useCallback(() => {
    dispatch({ "type": "reset" });
  }, [dispatch]);

  const onOperationChange = useCallback(
    (operation: Operation) => {
      resetInputs();
      setOperation(operation);
    },
    [resetInputs, setOperation]
  );

  const onSwitchSide = useCallback(() => {
    // setFocusedInput("market");
    resetInputs();
    setOperation(operation === Operation.Deposit ? Operation.Withdrawal : Operation.Deposit);
  }, [operation, resetInputs, setOperation]);

  const onMarketChange = useCallback(
    (marketAddress: string) => {
      resetInputs();
      onSelectMarket(marketAddress);
    },
    [onSelectMarket, resetInputs]
  );

  function onFocusedCollateralInputChange(
    tokenAddress: string
  ) {
    void tokenAddress;
    // if (!marketInfo) {
    //   return;
    // }

    // if (marketInfo.isSingle) {
    //   setFocusedInput("longCollateral");
    //   return;
    // }

    // if (getTokenPoolType(marketInfo, tokenAddress) === "long") {
    //   setFocusedInput("longCollateral");
    // } else {
    //   setFocusedInput("shortCollateral");
    // }
  }

  const availableModes = useMemo(() => getGmSwapBoxAvailableModes(operation, marketInfo), [operation, marketInfo]);
  const isDeposit = operation === Operation.Deposit;
  const isWithdrawal = operation === Operation.Withdrawal;
  const isSingle = mode === Mode.Single;
  const isPair = mode === Mode.Pair;

  const [indexName, setIndexName] = useLocalStorageSerializeKey<string>(
    getSyntheticsDepositIndexTokenKey(genesisHash),
    ""
  );

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

      <form
        onSubmit={(e) => {
          e.preventDefault();
          handleSubmit();
        }}
      >
        <div className={cx("GmSwapBox-form-layout", { reverse: isWithdrawal })}>
          <BuyInputSection
            topLeftLabel={isDeposit ? t`Pay` : t`Receive`}
            topLeftValue={formatUsd(firstTokenUsd)}
            // topRightLabel={t`Balance`}
            // topRightValue={formatTokenAmount(firstToken?.balance, firstToken?.decimals, "", {
            //   useCommas: true,
            // })}
            preventFocusOnLabelClick="right"
            // {...(isDeposit && {
            //   onClickTopRightLabel: onMaxClickFirstToken,
            // })}
            // showMaxButton={
            //   isDeposit &&
            //   firstToken?.balance?.gt(0) &&
            //   !firstTokenAmount?.eq(firstToken.balance) &&
            //   (firstToken?.isNative ? minResidualAmount && firstToken?.balance?.gt(minResidualAmount) : true)
            // }
            inputValue={inputState.firstTokenInputValue}
            onInputValueChange={(e) => {
              if (firstToken) {
                // setFirstTokenInputValue(e.target.value);
                dispatch({ type: "set-first-token-input-value", value: e.target.value });
                onFocusedCollateralInputChange(firstToken.address.toBase58());
              }
            }}
          // onClickMax={onMaxClickFirstToken}
          >
            {firstToken && isSingle ? (
              <TokenSelector
                label={isDeposit ? t`Pay` : t`Receive`}
                token={firstToken}
                onSelectToken={onSelectFirstToken}
                tokens={tokenOptions}
                // infoTokens={infoTokens}
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

          {isPair && secondToken && (
            <BuyInputSection
              topLeftLabel={isDeposit ? t`Pay` : t`Receive`}
              topLeftValue={formatUsd(secondTokenUsd)}
              // topRightLabel={t`Balance`}
              // topRightValue={formatTokenAmount(secondToken?.balance, secondToken?.decimals, "", {
              //   useCommas: true,
              // })}
              preventFocusOnLabelClick="right"
              inputValue={inputState.secondTokenInputValue}
              // showMaxButton={
              //   isDeposit &&
              //   secondToken?.balance?.gt(0) &&
              //   !secondTokenAmount?.eq(secondToken.balance) &&
              //   (secondToken?.isNative ? minResidualAmount && secondToken?.balance?.gt(minResidualAmount) : true)
              // }
              onInputValueChange={(e) => {
                if (secondToken) {
                  dispatch({ type: "set-second-token-input-value", value: e.target.value });
                  onFocusedCollateralInputChange(secondToken.address.toBase58());
                }
              }}
            // {...(isDeposit && {
            //   onClickTopRightLabel: onMaxClickSecondToken,
            // })}
            // onClickMax={onMaxClickSecondToken}
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
            topLeftValue={marketTokenUsd?.gt(BN_ZERO) ? formatUsd(marketTokenUsd) : ""}
            // topRightLabel={t`Balance`}
            // topRightValue={formatTokenAmount(marketToken?.balance, marketToken?.decimals, "", {
            //   useCommas: true,
            // })}
            preventFocusOnLabelClick="right"
            // showMaxButton={isWithdrawal && marketToken?.balance?.gt(0) && !marketTokenAmount?.eq(marketToken.balance)}
            inputValue={inputState.marketTokenInputValue}
            onInputValueChange={(e) => {
              dispatch({ type: "set-market-token-input-value", value: e.target.value });
              // setFocusedInput("market");
            }}
          // {...(isWithdrawal && {
          //   onClickTopRightLabel: () => {
          //     if (marketToken?.balance) {
          //       setMarketTokenInputValue(formatAmountFree(marketToken.balance, marketToken.decimals));
          //       setFocusedInput("market");
          //     }
          //   },
          // })}
          // onClickMax={() => {
          //   if (marketToken?.balance) {
          //     const formattedGMBalance = formatAmountFree(marketToken.balance, marketToken.decimals);
          //     const finalGMBalance = isMetamaskMobile
          //       ? limitDecimals(formattedGMBalance, MAX_METAMASK_MOBILE_DECIMALS)
          //       : formattedGMBalance;
          //     setMarketTokenInputValue(finalGMBalance);
          //     setFocusedInput("market");
          //   }
          // }}
          >
            <PoolSelector
              label={t`Pool`}
              className="SwapBox-info-dropdown"
              selectedIndexName={indexName}
              selectedMarketAddress={marketInfo.marketTokenAddress.toBase58()}
              markets={sortedMarketsInfoByIndexToken}
              marketTokensData={marketTokens}
              isSideMenu
              showBalances
              showAllPools
              showIndexIcon
              onSelectMarket={(marketInfo) => {
                setIndexName(getMarketIndexName(marketInfo));
                onMarketChange(marketInfo.marketTokenAddress.toBase58());
                // showMarketToast(marketInfo);
              }}
            />
          </BuyInputSection>
        </div>

        {/* <ExchangeInfo className="GmSwapBox-info-section" dividerClassName="App-card-divider">
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
        </ExchangeInfo> */}

        <div className="Exchange-swap-button-container">
          <Button
            className="w-full"
            variant="primary-action"
            type="submit"
          // onClick={submitState.onSubmit}
          // disabled={submitState.isDisabled}
          >
            {isDeposit ? t`Buy GM` : t`Sell GM`}
          </Button>
        </div>
        {/* <GmConfirmationBox
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
      </form>
    </div>
  );
}
