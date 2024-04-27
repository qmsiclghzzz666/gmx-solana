import Tab from "@/components/Tab/Tab";
import { TokenOptions, getGmSwapBoxAvailableModes } from "../utils";
import { CreateDepositParams, CreateWithdrawalParams, Mode, Operation } from "../types";
import { useCallback, useMemo } from "react";
import { useLingui } from "@lingui/react";
import { mapValues } from "lodash";
import cx from "classnames";
import { Trans, t } from "@lingui/macro";

import "./GmSwapBox.scss";
import { formatUsd, getMarketIndexName, getMarketPoolName } from "@/components/MarketsList/utils";
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
import { useGmInputAmounts, useGmInputDisplay, useGmStateDispath, useGmStateSelector, useHandleSubmit } from "../hooks";
import { formatAmountFree, formatTokenAmount } from "@/utils/number";
import { useInitializeTokenAccount } from "@/onchain/token";
import { PublicKey } from "@solana/web3.js";
import { useOpenConnectModal } from "@/contexts/anchor";
import { useNativeTokenUtils } from "@/components/NativeTokenUtils";
import { MarketInfo } from "@/onchain/market";
import { helperToast } from "@/utils/helperToast";
import LoadingDots from "@/components/Common/LoadingDots/LoadingDots";

const OPERATION_LABELS = {
  [Operation.Deposit]: /*i18n*/ "Buy GM",
  [Operation.Withdrawal]: /*i18n*/ "Sell GM",
};

const MODE_LABELS = {
  [Mode.Single]: /*i18n*/ "Single",
  [Mode.Pair]: /*i18n*/ "Pair",
};

export function GmForm({
  owner,
  genesisHash,
  tokenOptions: { tokenOptions, firstToken, secondToken },
  isPending,
  setOperation,
  setMode,
  onSelectMarket,
  onSelectFirstToken,
  onCreateDeposit,
  onCreateWithdrawal,
}: {
  owner: PublicKey | undefined,
  genesisHash: string,
  tokenOptions: TokenOptions,
  isPending: boolean,
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
    marketToken,
    marketTokens,
    sortedMarketsInfoByIndexToken,
  } = useGmStateSelector(s => {
    return {
      inputState: s.input,
      marketInfo: s.market,
      marketTokens: s.marketTokens,
      marketToken: s.marketToken,
      sortedMarketsInfoByIndexToken: s.sortedMarketsInfoByIndexToken,
    }
  });
  const { marketTokenAmount, firstTokenAmount, secondTokenAmount } = useGmInputAmounts();
  const { firstTokenUsd, secondTokenUsd, marketTokenUsd } = useGmInputDisplay();
  const { operation, mode } = useGmStateSelector(s => {
    return {
      operation: s.operation,
      mode: s.mode,
    };
  });

  const { isNativeTokenReady, openWrapNativeTokenModal } = useNativeTokenUtils();

  // const [focusedInput, setFocusedInput] = useState<"longCollateral" | "shortCollateral" | "market">("market");

  const resetInputs = useCallback(() => {
    dispatch({ "type": "reset" });
  }, [dispatch]);

  const performAction = useHandleSubmit({ onCreateDeposit, onCreateWithdrawal });
  const openConnectModal = useOpenConnectModal();

  const handleSubmit = useCallback(() => {
    if (owner) {
      performAction();
    } else {
      openConnectModal();
    }
    resetInputs();
  }, [owner, performAction, resetInputs, openConnectModal]);

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
  const isMarketTokenAccountInited = marketToken?.balance !== null;
  const isFirstTokenAccountInited = firstToken?.balance !== null;
  const isSecondTokenAccountInited = secondToken?.balance !== null;
  const allowWrapFirstToken = owner && isNativeTokenReady && isDeposit && isFirstTokenAccountInited && firstToken?.isWrappedNative;
  const allowWrapSecondToken = owner && isNativeTokenReady && isDeposit && isSecondTokenAccountInited && secondToken?.isWrappedNative;

  const [indexName, setIndexName] = useLocalStorageSerializeKey<string>(
    getSyntheticsDepositIndexTokenKey(genesisHash),
    ""
  );

  function onMaxClickFirstToken() {
    if (firstToken?.balance) {
      let maxAvailableAmount = firstToken.balance;

      if (maxAvailableAmount.isNeg()) {
        maxAvailableAmount = BN_ZERO;
      }

      const formattedMaxAvailableAmount = formatAmountFree(maxAvailableAmount, firstToken.decimals);
      const finalAmount = formattedMaxAvailableAmount;

      dispatch({ type: "set-first-token-input-value", value: finalAmount });
      // onFocusedCollateralInputChange(firstToken.address);
    }
  }

  function onMaxClickSecondToken() {
    if (secondToken?.balance) {
      let maxAvailableAmount = secondToken.balance;

      if (maxAvailableAmount.isNeg()) {
        maxAvailableAmount = BN_ZERO
      }

      const formattedMaxAvailableAmount = formatAmountFree(maxAvailableAmount, secondToken.decimals);
      const finalAmount = formattedMaxAvailableAmount;

      dispatch({ type: "set-second-token-input-value", value: finalAmount });
      // onFocusedCollateralInputChange(secondToken.address);
    }
  }

  const { isSending: isInitializing, trigger: initializeTokenAccount } = useInitializeTokenAccount();

  const wrapNativeToken = useCallback(() => {
    openWrapNativeTokenModal();
  }, [openWrapNativeTokenModal]);

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
            topLeftLabel={isDeposit ? (allowWrapFirstToken ? t`Pay (wrap)` : t`Pay`) : t`Receive`}
            {...(allowWrapFirstToken && {
              onClientTopLeftLabel: wrapNativeToken,
            })}
            topLeftValue={formatUsd(firstTokenUsd)}
            topRightLabel={isFirstTokenAccountInited ? t`Balance` : isInitializing ? t`Initializing` : t`Uninitialized`}
            topRightValue={isFirstTokenAccountInited ? formatTokenAmount(firstToken?.balance || BN_ZERO, firstToken?.decimals, "", {
              useCommas: true,
            }) : ""}
            preventFocusOnLabelClick="right"
            {...(isDeposit && isFirstTokenAccountInited && {
              onClickTopRightLabel: onMaxClickFirstToken,
            })}
            {...(!isFirstTokenAccountInited && !isInitializing && {
              onClickTopRightLabel: () => void initializeTokenAccount(firstToken.address),
            })}
            onClickMax={onMaxClickFirstToken}
            showMaxButton={
              isDeposit &&
              firstToken?.balance?.gt(BN_ZERO) &&
              !firstTokenAmount?.eq(firstToken.balance)
            }
            inputValue={inputState.firstTokenInputValue}
            onInputValueChange={(e) => {
              if (firstToken) {
                // setFirstTokenInputValue(e.target.value);
                dispatch({ type: "set-first-token-input-value", value: e.target.value });
                onFocusedCollateralInputChange(firstToken.address.toBase58());
              }
            }}
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
              topLeftLabel={isDeposit ? allowWrapSecondToken ? t`Pay (wrap)` : t`Pay` : t`Receive`}
              {...(allowWrapSecondToken && {
                onClientTopLeftLabel: wrapNativeToken,
              })}
              topLeftValue={formatUsd(secondTokenUsd)}
              topRightLabel={isSecondTokenAccountInited ? t`Balance` : isInitializing ? t`Initializing...` : t`Uninitialized`}
              topRightValue={isSecondTokenAccountInited ? formatTokenAmount(secondToken?.balance ?? BN_ZERO, secondToken?.decimals, "", {
                useCommas: true,
              }) : ""}
              preventFocusOnLabelClick="right"
              inputValue={inputState.secondTokenInputValue}
              showMaxButton={
                isDeposit &&
                secondToken?.balance?.gt(BN_ZERO) &&
                !secondTokenAmount?.eq(secondToken.balance)
              }
              onInputValueChange={(e) => {
                if (secondToken) {
                  dispatch({ type: "set-second-token-input-value", value: e.target.value });
                  onFocusedCollateralInputChange(secondToken.address.toBase58());
                }
              }}
              {...(isDeposit && isSecondTokenAccountInited && {
                onClickTopRightLabel: onMaxClickSecondToken,
              })}
              {...(!isSecondTokenAccountInited && !isInitializing && {
                onClickTopRightLabel: () => void initializeTokenAccount(secondToken.address),
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
            topLeftValue={marketTokenUsd?.gt(BN_ZERO) ? formatUsd(marketTokenUsd) : ""}
            topRightLabel={isMarketTokenAccountInited ? t`Balance` : isInitializing ? t`Initializing` : t`Uninitialized`}
            topRightValue={isMarketTokenAccountInited ? formatTokenAmount(marketToken?.balance ?? BN_ZERO, marketToken?.decimals, "", {
              useCommas: true,
            }) : ""}
            preventFocusOnLabelClick="right"
            showMaxButton={isWithdrawal && marketToken?.balance?.gt(BN_ZERO) && !marketTokenAmount?.eq(marketToken.balance)}
            inputValue={inputState.marketTokenInputValue}
            onInputValueChange={(e) => {
              dispatch({ type: "set-market-token-input-value", value: e.target.value });
              // setFocusedInput("market");
            }}
            {...(isMarketTokenAccountInited && isWithdrawal && {
              onClickTopRightLabel: () => {
                if (marketToken?.balance) {
                  dispatch({
                    type: "set-market-token-input-value",
                    value: formatAmountFree(marketToken.balance, marketToken.decimals)
                  });
                  // setFocusedInput("market");
                }
              },
            })}
            {...(!isMarketTokenAccountInited && !isInitializing && {
              onClickTopRightLabel: () => void initializeTokenAccount(marketToken.address),
            })}
            onClickMax={() => {
              if (marketToken?.balance) {
                const formattedGMBalance = formatAmountFree(marketToken.balance, marketToken.decimals);
                dispatch({
                  type: "set-market-token-input-value",
                  value: formattedGMBalance
                });
                // setFocusedInput("market");
              }
            }}
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
                showMarketToast(marketInfo);
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
            disabled={isPending}
          // onClick={submitState.onSubmit}
          // disabled={submitState.isDisabled}
          >
            {isPending && <LoadingDots size={14} />}
            {!isPending && (owner ? isDeposit ? t`Buy GM` : t`Sell GM` : t`Connect Wallet`)}
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
      </form >
    </div >
  );
}

function showMarketToast(market: MarketInfo) {
  if (!market) return;
  const indexName = getMarketIndexName(market);
  const poolName = getMarketPoolName(market);
  helperToast.success(
    <Trans>
      <div className="inline-flex">
        GM:&nbsp;<span>{indexName}</span>
        <span className="subtext gm-toast">[{poolName}]</span>
      </div>{" "}
      <span>selected in order form</span>
    </Trans>
  );
}
