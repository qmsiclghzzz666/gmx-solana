import { PositionInfo } from "@/onchain/position";
import { TradeMode } from "@/onchain/trade";
import { Trans, t } from "@lingui/macro";
import { PositionItem } from "./PositionItem";
import { useSharedStatesSelector } from "@/contexts/shared";
import { selectIsPositionLoading, selectPositionList } from "@/contexts/shared/selectors/position-selectors";

type Props = {
  onSelectPositionClick: (key: string, tradeMode?: TradeMode) => void;
  onClosePositionClick: (key: string) => void;
  onSettlePositionFeesClick: (key: string) => void;
  onOrdersClick: (key?: string) => void;
  openSettings: () => void;
  hideActions?: boolean;
};

function usePositions(): PositionInfo[] {
  return useSharedStatesSelector(selectPositionList);
}

function useIsPositionLoading(): boolean {
  return useSharedStatesSelector(selectIsPositionLoading);
}

export function PositionList({
  onClosePositionClick,
  onOrdersClick,
  onSelectPositionClick,
  onSettlePositionFeesClick,
  openSettings,
  hideActions,
}: Props) {
  const positions = usePositions();
  const isLoading = useIsPositionLoading();
  return (
    <>
      {positions.length === 0 && (
        <div className="Exchange-empty-positions-list-note App-card small">
          {isLoading ? t`Loading...` : t`No open positions`}
        </div>
      )}

      <div className="Exchange-list small">
        {!isLoading &&
          positions.map((position) => (
            // <PositionItemWrapper
            //   key={position.key}
            //   position={position}
            //   onEditCollateralClick={setEditingPositionKey}
            //   onClosePositionClick={onClosePositionClick}
            //   onGetPendingFeesClick={onSettlePositionFeesClick}
            //   onOrdersClick={onOrdersClick}
            //   onSelectPositionClick={onSelectPositionClick}
            //   isLarge={false}
            //   onShareClick={handleSharePositionClick}
            //   openSettings={openSettings}
            //   hideActions={hideActions}
            // />
            <PositionItem
              key={position.address.toBase58()}
              position={position}
              isLarge={false}
              onClosePositionClick={onClosePositionClick}
            />
          ))}
      </div>

      <table className="Exchange-list Position-list large App-box">
        <tbody>
          <tr className="Exchange-list-header">
            <th>
              <Trans>Position</Trans>
            </th>
            <th>
              <Trans>Net Value</Trans>
            </th>
            <th>
              <Trans>Size</Trans>
            </th>
            <th>
              <Trans>Collateral</Trans>
            </th>
            <th>
              <Trans>Entry Price</Trans>
            </th>
            <th>
              <Trans>Mark Price</Trans>
            </th>
            <th>
              <Trans>Liq. Price</Trans>
            </th>
          </tr>
          {positions.length === 0 && (
            <tr>
              <td colSpan={15}>
                <div className="Exchange-empty-positions-list-note">
                  {isLoading ? t`Loading...` : t`No open positions`}
                </div>
              </td>
            </tr>
          )}
          {!isLoading &&
            positions.map((position) => (
              <PositionItem
                key={position.address.toBase58()}
                position={position}
                isLarge
                onClosePositionClick={onClosePositionClick}
              />
            ))}
        </tbody>
      </table>
    </>
  );
}
