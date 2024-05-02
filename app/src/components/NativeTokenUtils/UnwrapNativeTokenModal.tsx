import { BN_ZERO } from "@/config/constants";
import { useUnwrapNativeToken } from "@/onchain/token";
import { TokenData } from "@/onchain/token";
import { convertToUsd, formatTokenAmount } from "@/utils/number";
import { useCallback } from "react";
import Modal from "../Modal/Modal";
import { t } from "@lingui/macro";
import BuyInputSection from "../BuyInputSection/BuyInputSection";
import { formatUsd } from "../MarketsList/utils";
import TokenWithIcon from "../TokenIcon/TokenWithIcon";
import Button from "../Button/Button";

import "./UnwrapNativeTokenModal.scss";
import { useAnchor } from "@/contexts/anchor";
import LoadingDots from "../Common/LoadingDots/LoadingDots";

export function UnwrapNativeTokenModal({
  isVisible,
  wrappedNativeToken,
  onSubmitted,
  onClose,
}: {
  isVisible: boolean,
  wrappedNativeToken: TokenData,
  onSubmitted: () => void,
  onClose: () => void,
}) {
  const { active } = useAnchor();
  const handleSubmitted = useCallback(() => {
    onSubmitted();
  }, [onSubmitted]);

  const { trigger: unwrapNativeToken, isSending } = useUnwrapNativeToken(handleSubmitted);

  const handleSubmit = useCallback(() => {
    void unwrapNativeToken(undefined);
  }, [unwrapNativeToken]);

  const isInitilized = wrappedNativeToken.balance !== null;
  const allowToUnwrap = active && isInitilized;

  const wrappedNativeTokenUsd = convertToUsd(wrappedNativeToken.balance ?? BN_ZERO, wrappedNativeToken.decimals, wrappedNativeToken.prices.minPrice);

  const inputValue = formatTokenAmount(wrappedNativeToken.balance ?? BN_ZERO, wrappedNativeToken?.decimals, "");

  return (
    <Modal
      className="unwrap-native-token-modal"
      isVisible={isVisible}
      onClose={() => {
        onClose();
      }} label={t`Unwrap WSOL`}>
      <form onSubmit={(e) => {
        e.preventDefault();
        handleSubmit();
      }}>
        <BuyInputSection
          topLeftLabel={t`Value`}
          topLeftValue={wrappedNativeTokenUsd?.gt(BN_ZERO) ? formatUsd(wrappedNativeTokenUsd) : ""}
          topRightLabel={isInitilized ? t`Balance` : t`Uninitialized`}
          topRightValue={isInitilized ? formatTokenAmount(wrappedNativeToken.balance ?? BN_ZERO, wrappedNativeToken?.decimals, "", {
            useCommas: true,
          }) : ""}
          inputValue={inputValue}
          staticInput
        >
          <div className="selected-token">
            <TokenWithIcon symbol={wrappedNativeToken.symbol} displaySize={20} />
          </div>
        </BuyInputSection>

        <Button
          disabled={!allowToUnwrap || isSending}
          className="w-full"
          variant="primary-action"
          type="submit"
        >
          {isSending ? <LoadingDots size={14} /> : t`Unwrap`}
        </Button>
      </form>
    </Modal>
  );
}
