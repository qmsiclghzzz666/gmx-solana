import { t } from "@lingui/macro";
import ExchangeInfoRow from "../Exchange/ExchangeInfoRow";
import TokenSelector from "../TokenSelector/TokenSelector";
import { useMemo } from "react";
import { useSharedStatesSelector } from "@/contexts/shared";
import { Tokens } from "@/onchain/token";
import { getPoolUsdWithoutPnl } from "@/onchain/market";
import { selectMarketInfo, selectCollateralToken, selectSetCollateralAddress } from "@/contexts/shared/selectors/trade-box-selectors";

export function CollateralSelectorRow() {
  const marketInfo = useSharedStatesSelector(selectMarketInfo);
  const selectedCollateralToken = useSharedStatesSelector(selectCollateralToken);
  const setCollateralAddress = useSharedStatesSelector(selectSetCollateralAddress);

  const {
    allRelatedTokensArr,
    allRelatedTokensMap,
  } = useMemo(() => {
    if (!marketInfo) return { allRelatedTokensArr: [], allRelatedTokensMap: {} }

    const allRelatedTokensMap: Tokens = {};

    allRelatedTokensMap[marketInfo.longTokenAddress.toBase58()] = marketInfo.longToken;
    allRelatedTokensMap[marketInfo.shortTokenAddress.toBase58()] = marketInfo.shortToken;

    const allRelatedTokensArr = Object.values(allRelatedTokensMap).sort((a, b) => {
      const aIsLong = a.address.equals(marketInfo.longToken.address);
      const bIsLong = b.address.equals(marketInfo.shortToken.address);
      const aLiquidity = getPoolUsdWithoutPnl(marketInfo, aIsLong, "minPrice");
      const bLiquidity = getPoolUsdWithoutPnl(marketInfo, bIsLong, "midPrice");
      return aLiquidity.gte(bLiquidity) ? -1 : 1;
    });
    return {
      allRelatedTokensArr,
      allRelatedTokensMap,
    }
  }, [marketInfo]);

  return (
    <>
      <ExchangeInfoRow
        label={t`Collateral In`}
        className="SwapBox-info-row"
        value={
          selectedCollateralToken &&
          allRelatedTokensArr.length !== 0 && (
            <TokenSelector
              label={t`Collateral In`}
              className="GlpSwap-from-token SwapBox-info-dropdown"
              token={selectedCollateralToken}
              onSelectToken={(token) => {
                setCollateralAddress(token.address.toBase58());
              }}
              tokens={allRelatedTokensArr}
              infoTokens={allRelatedTokensMap}
              showTokenImgInDropdown={true}
            // getTokenState={getTokenState}
            />
          )
        }
      />
      {/* {messages} */}
    </>
  );
}
