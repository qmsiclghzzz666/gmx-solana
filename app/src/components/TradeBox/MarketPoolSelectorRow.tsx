import { useSharedStatesSelector } from "@/contexts/shared";
import { MarketInfo } from "@/onchain/market";
import { Token } from "@/onchain/token";
import { BN } from "@coral-xyz/anchor";
import { getMarketIndexName } from "../MarketsList/utils";
import ExchangeInfoRow from "../Exchange/ExchangeInfoRow";
import { t } from "@lingui/macro";
import { PoolSelector } from "../MarketSelector/PoolSelector";
import { selectAvailableMarkets } from "@/contexts/shared/selectors/trade-box-selectors";

type Props = {
  indexToken?: Token;
  selectedMarket?: MarketInfo;
  isOutPositionLiquidity?: boolean;
  currentPriceImpactBps?: BN;
  onSelectMarketAddress: (marketAddress?: string) => void;
};

export function MarketPoolSelectorRow(p: Props) {
  const { selectedMarket, indexToken, onSelectMarketAddress } = p;
  const availableMarkets = useSharedStatesSelector(selectAvailableMarkets);
  const indexName = indexToken ? getMarketIndexName({ indexToken, isSpotOnly: false }) : undefined;
  return (
    <>
      <ExchangeInfoRow
        className="SwapBox-info-row"
        label={t`Pool`}
        value={
          <>
            <PoolSelector
              label={t`Pool`}
              className="SwapBox-info-dropdown"
              selectedIndexName={indexName}
              selectedMarketAddress={selectedMarket?.marketTokenAddress.toBase58()}
              markets={availableMarkets ?? []}
              isSideMenu
              onSelectMarket={(marketInfo) => onSelectMarketAddress(marketInfo.marketTokenAddress.toBase58())}
            />
          </>
        }
      />

      {/* <TradeboxPoolWarnings /> */}
    </>
  );
}
