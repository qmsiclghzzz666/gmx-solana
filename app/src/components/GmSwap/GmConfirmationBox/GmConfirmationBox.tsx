import "./GmConfirmationBox.scss";
import Button from "@/components/Button/Button";
import LoadingDots from "@/components/Common/LoadingDots/LoadingDots";
import { withInitializeTokenAccountGuard } from "@/components/InitializeTokenAccountGuard";
import Modal from "@/components/Modal/Modal";
import { Trans, t } from "@lingui/macro";
import { useCallback, useState } from "react";
import { useGmStateSelector } from "../hooks";
import { selectIsDeposit, selectMarket, selectParams } from "../selectors";
// import { FaArrowRight } from "react-icons/fa";
import { TokenData } from "@/onchain/token";
import { formatAmount, formatUsd, getMarketIndexName } from "@/components/MarketsList/utils";
import { BN } from "@coral-xyz/anchor";
import { BN_ZERO } from "@/config/constants";
import TokenWithIcon from "@/components/TokenIcon/TokenWithIcon";
import { MarketInfo } from "@/onchain/market";
import CheckBox from "@/components/Common/CheckBox/CheckBox";

interface Props {
  isPending: boolean,
  isVisible: boolean,
  operationText: string,
  onClose: () => void,
  onSubmit: (skipPreflight: boolean) => Promise<void>,
  onSubmitted?: () => void,
}

export const GmConfirmationBox = withInitializeTokenAccountGuard(GmConfirmationBoxInner);

function GmConfirmationBoxInner({
  isPending,
  isVisible,
  operationText,
  onClose,
  onSubmit,
  onSubmitted,
}: Props) {
  const isDeposit = useGmStateSelector(selectIsDeposit);
  const market = useGmStateSelector(selectMarket);
  const {
    tokens,
    amounts,
    display
  } = useGmStateSelector(selectParams);

  const [skipPreflight, setSkipPreflight] = useState(false);

  const handleSubmit = useCallback(() => {
    void onSubmit(skipPreflight).then(onSubmitted);
  }, [onSubmit, onSubmitted, skipPreflight]);
  return (
    <div className="Confirmation-box GmConfirmationBox">
      <Modal isVisible={isVisible} onClose={onClose} label={t`Confirm ${operationText}`}>
        {isVisible && (
          <>
            {isDeposit && (
              <div className="Confirmation-box-main trade-info-wrapper">
                <div className="trade-info">
                  <Trans>Pay</Trans>{" "}
                  {market?.isSingle ? (
                    renderTokenInfo({
                      amount: amounts.firstTokenAmount.add(amounts.secondTokenAmount),
                      usd: display.firstTokenUsd?.add(display.secondTokenUsd ?? BN_ZERO),
                      token: tokens.firstToken,
                    })
                  ) : (
                    <>
                      {renderTokenInfo({
                        amount: amounts.firstTokenAmount,
                        usd: display.firstTokenUsd,
                        token: tokens.firstToken,
                      })}
                      {renderTokenInfo({
                        amount: amounts.secondTokenAmount,
                        usd: display.secondTokenUsd,
                        token: tokens.secondToken,
                        // overrideSymbol: shortSymbol,
                        className: "mt-xs",
                      })}
                    </>
                  )}
                </div>
                {/* <FaArrowRight className="arrow-icon" fontSize={12} color="#ffffffb3" />
                <div className="trade-info">
                  <Trans>Receive</Trans>{" "}
                  {renderTokenInfo({
                    amount: marketTokenAmount,
                    usd: marketTokenUsd,
                    token: marketToken,
                  })}
                </div> */}
              </div>
            )}
            {!isDeposit && (
              <div className="Confirmation-box-main trade-info-wrapper">
                <div className="trade-info">
                  <Trans>Pay</Trans>{" "}
                  {renderTokenInfo({
                    amount: amounts.marketTokenAmount,
                    usd: display.marketTokenUsd,
                    token: tokens.marketToken,
                    market,
                  })}
                </div>
                {/* <FaArrowRight className="arrow-icon" fontSize={12} color="#ffffffb3" />
                <div className="trade-info">
                  <Trans>Receive</Trans>{" "}
                  {market?.isSingle ? (
                    renderTokenInfo({
                      amount: longTokenAmount?.add(shortTokenAmount!),
                      usd: longTokenUsd?.add(shortTokenUsd!),
                      token: longToken,
                    })
                  ) : (
                    <>
                      {renderTokenInfo({
                        amount: longTokenAmount,
                        usd: longTokenUsd,
                        token: longToken,
                        overrideSymbol: longSymbol,
                      })}
                      {renderTokenInfo({
                        amount: shortTokenAmount,
                        usd: shortTokenUsd,
                        token: shortToken,
                        overrideSymbol: shortSymbol,
                        className: "mt-xs",
                      })}
                    </>
                  )}
                </div> */}
              </div>
            )}
            <CheckBox isChecked={skipPreflight} setIsChecked={setSkipPreflight}>
              <span className="muted font-sm">
                <Trans>Skip transaction preflight.</Trans>
              </span>
            </CheckBox>
            <div className="Confirmation-box-row">
              <Button
                className="w-full"
                variant="primary-action"
                onClick={handleSubmit}
                disabled={isPending}
              >
                {isPending ? <LoadingDots /> : t`Confirm`}
              </Button>
            </div>
          </>
        )}
      </Modal>
    </div>
  );
}

function renderTokenInfo({
  amount,
  className,
  token,
  usd,
  market,
}: {
  amount?: BN;
  usd?: BN;
  token?: TokenData;
  className?: string;
  market?: MarketInfo,
}) {
  if (!amount || !usd || !token) return;
  return (
    <div className={className ?? ""}>
      <div className="trade-token-amount">
        <span>
          {formatAmount(amount, token?.decimals, 4, true)}{" "}
          {!market && <TokenWithIcon symbol={token.symbol} displaySize={20} />}
          {market && <TokenWithIcon symbol={market.indexToken.symbol} displaySize={20} name={`GM:${getMarketIndexName(market)}`} />}
        </span>
      </div>
      <div className="trade-amount-usd">{formatUsd(usd)}</div>
    </div>
  );
}
