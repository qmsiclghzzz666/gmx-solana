import "./PositionItem.scss";
import { PositionInfo } from "@/onchain/position";

export function PositionItem({ position }: { position: PositionInfo }) {
  return (
    <div>
      {`${position.address.toBase58()}`}
    </div>
  );
}
