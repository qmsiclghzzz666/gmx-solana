import { useEffect, useState } from "react";
import { useDataStore } from "../contexts/anchor";
import "./Dashboard.css";

import { findMarketPDA } from "gmsol";
import { getGMSOLDeployment } from "../config/deployment";
import { PublicKey } from "@solana/web3.js";

interface Market {
  long?: string,
  short?: string,
  pubkey?: PublicKey,
}

export default function Dashboard() {
  const dataStore = useDataStore();
  const [market, setMarket] = useState<Market | null>(null);

  useEffect(() => {
    const fetchMarket = async () => {
      const deployment = getGMSOLDeployment();

      if (deployment) {
        const [address] = findMarketPDA(deployment.store, deployment.markets[0].market_token);
        const data = await dataStore?.account.market.fetch(address);
        setMarket({
          long: data?.pools.pools[0].longTokenAmount.toString(),
          short: data?.pools.pools[0].shortTokenAmount.toString(),
          pubkey: dataStore?.provider.publicKey,
        });
      }
    };

    void fetchMarket();
  }, [dataStore]);

  return (
    <div className="default-container page-layout">
      <div>long: {market?.long}</div>
      <div>short: {market?.short}</div>
      <div>connected: {market?.pubkey?.toBase58() ?? ""} </div>
    </div>
  )
}
