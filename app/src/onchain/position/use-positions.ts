import { useAnchor, useDataStore } from "@/contexts/anchor";
import { Address, translateAddress } from "@coral-xyz/anchor";
import { useMemo } from "react";
import { findPositionPDA } from "gmsol";
import { Market } from "../market";
import useSWR from "swr";
import { Position, Positions } from "./types";
import { PublicKey } from "@solana/web3.js";

const POSITIONS_KEY = "data_store/positions";

export const usePositions = (params?: { store: Address, markets: Market[] }) => {
  const dataStore = useDataStore();
  const { owner } = useAnchor();
  const request = useMemo(() => {
    return params && owner ? {
      key: POSITIONS_KEY,
      owner: owner.toBase58(),
      positionAddresses: params.markets.flatMap(market => {
        const storeAddress = translateAddress(params.store);
        return Array.from(new Set([
          findPositionPDA(storeAddress, owner, market.marketTokenAddress, market.longTokenAddress, true)[0].toBase58(),
          findPositionPDA(storeAddress, owner, market.marketTokenAddress, market.longTokenAddress, false)[0].toBase58(),
          findPositionPDA(storeAddress, owner, market.marketTokenAddress, market.shortTokenAddress, true)[0].toBase58(),
          findPositionPDA(storeAddress, owner, market.marketTokenAddress, market.shortTokenAddress, false)[0].toBase58(),
        ]).values());
      }),
    } : null;
  }, [owner, params]);

  const { data, isLoading } = useSWR(request, async ({ positionAddresses, owner }) => {
    const data = await dataStore.account.position.fetchMultiple(positionAddresses);
    const positions = data.filter(position => position).map((position, idx) => {
      const address = positionAddresses[idx];
      if (!(position!.kind === 1 || position!.kind === 2)) return;
      return {
        address: new PublicKey(address),
        owner: new PublicKey(owner),
        marketTokenAddress: position!.marketToken,
        collateralTokenAddress: position!.collateralToken,
        isLong: position!.kind === 1,
        sizeInUsd: position!.sizeInUsd,
        sizeInTokens: position!.sizeInTokens,
        collateralAmount: position!.collateralAmount,
      } satisfies Position as Position;
    });
    return positions.reduce((acc, position) => {
      if (position) {
        acc[position.address.toBase58()] = position;
      }
      return acc;
    }, {} as Positions);
  });

  return {
    positions: data ?? {},
    isLoading,
  }
};
