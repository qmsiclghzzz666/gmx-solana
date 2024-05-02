import { useCallback, useMemo } from "react";
import Modal from "../Modal/Modal";
import "./PositionSeller.scss";
import { Trans, t } from "@lingui/macro";
import BuyInputSection from "../BuyInputSection/BuyInputSection";
import Button from "../Button/Button";
import { useClearClosingPosition, useClosingPosition } from "@/contexts/shared";
import { BN_ZERO, USD_DECIMALS } from "@/config/constants";
import { formatUsd } from "../MarketsList/utils";
import { formatAmountFree, parseValue, toBigInt } from "@/utils/number";
import { useDebounceValue } from "usehooks-ts";
import { DEBOUNCE_MS } from "@/config/ui";
import { useTriggerInvocation } from "@/onchain/transaction";
import LoadingDots from "../Common/LoadingDots/LoadingDots";
import { useSWRConfig } from "swr";
import { filterBalances } from "@/onchain/token";
import { fitlerMarkets } from "@/onchain/market";
import { fitlerPositions } from "@/onchain/position";
import { MakeCreateDecreaseOrderParams, invokeCreateDecreaseOrder } from "gmsol";
import { useExchange, useOpenConnectModal } from "@/contexts/anchor";
import { GMSOL_DEPLOYMENT } from "@/config/deployment";

export function PositionSeller() {
  const position = useClosingPosition();
  const [closeUsdInputValue, setCloseUsdInputValue] = useDebounceValue("", DEBOUNCE_MS);
  const resetInputs = useCallback(() => {
    setCloseUsdInputValue("");
  }, [setCloseUsdInputValue]);
  const clearClosingPosition = useClearClosingPosition();
  const handleClose = useCallback(() => {
    clearClosingPosition();
    resetInputs();
  }, [clearClosingPosition, resetInputs]);

  const { mutate } = useSWRConfig();

  const mutateStates = useCallback(() => {
    void mutate(filterBalances);
    void mutate(fitlerMarkets);
    void mutate(fitlerPositions);
  }, [mutate]);

  const exchange = useExchange();
  const payer = exchange.provider.publicKey;
  const openConnectModal = useOpenConnectModal();

  const createDecreaseOrder = useCallback(async (params: MakeCreateDecreaseOrderParams) => {
    if (payer) {
      const [signature, order] = await invokeCreateDecreaseOrder(exchange, params, { skipPreflight: false });
      console.log(`created a decrease order ${order.toBase58()} at tx ${signature}`);
      return signature;
    } else {
      throw Error("Wallet is not connected");
    }
  }, [exchange, payer]);

  const { trigger, isSending } = useTriggerInvocation({
    key: "exchange-create-decrease-order",
    onSentMessage: t`Creating decrease order...`,
    message: t`Decrease order created.`,
  }, createDecreaseOrder, { onSuccess: mutateStates });

  const isVisible = Boolean(position);
  const maxCloseSize = position?.sizeInUsd ?? BN_ZERO;
  const closeSizeUsd = useMemo(() => parseValue(closeUsdInputValue || "0", USD_DECIMALS)!, [closeUsdInputValue]);

  const handleSubmit = useCallback(() => {
    if (!payer) {
      openConnectModal();
      return;
    }
    if (!position) return handleClose();
    void trigger({
      store: GMSOL_DEPLOYMENT!.store,
      payer,
      position: position.address,
      sizeDeltaUsd: toBigInt(closeSizeUsd),
      options: {
        hint: {
          market: {
            marketToken: position.marketInfo.marketTokenAddress,
            longToken: position.marketInfo.longToken.address,
            shortToken: position.marketInfo.shortToken.address,
          },
          collateralToken: position.collateralToken.address,
          isLong: position.isLong,
        }
      }
    }).then(handleClose);
  }, [closeSizeUsd, handleClose, openConnectModal, payer, position, trigger]);

  return (
    <div className="PositionEditor PositionSeller">
      <Modal
        className="PositionSeller-modal"
        isVisible={isVisible}
        onClose={handleClose}
        label={
          <Trans>
            Close {position?.isLong ? t`Long` : t`Short`} {position?.marketInfo.indexToken?.symbol}
          </Trans>
        }
      >
        {position && (
          <>
            <div className="relative">
              <BuyInputSection
                topLeftLabel={t`Close`}
                topRightLabel={t`Max`}
                topRightValue={formatUsd(maxCloseSize)}
                inputValue={closeUsdInputValue}
                onInputValueChange={(e) => setCloseUsdInputValue(e.target.value)}
                showMaxButton={maxCloseSize?.gt(BN_ZERO) && !closeSizeUsd?.eq(maxCloseSize)}
                onClickMax={() => setCloseUsdInputValue(formatAmountFree(maxCloseSize, USD_DECIMALS))}
                showPercentSelector
                onPercentChange={(percentage) => {
                  const formattedAmount = formatAmountFree(maxCloseSize.muln(percentage).divn(100), USD_DECIMALS, 2);
                  setCloseUsdInputValue(formattedAmount);
                }}
              >
                USD
              </BuyInputSection>
            </div>
            <div className="Exchange-swap-button-container">
              <Button
                className="w-full"
                variant="primary-action"
                disabled={isSending}
                onClick={handleSubmit}
              // buttonRef={submitButtonRef}
              >
                {/* {error ||
                  (isTrigger
                    ? t`Create ${getTriggerNameByOrderType(decreaseAmounts?.triggerOrderType)} Order`
                    : t`Close`)} */}
                {isSending ? <LoadingDots /> : t`Close`}
              </Button>
            </div>
          </>
        )}
      </Modal>
    </div>
  );
}
