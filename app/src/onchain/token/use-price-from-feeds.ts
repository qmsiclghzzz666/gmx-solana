import { DEFAULT_CLUSTER } from "@/config/env";
import { PythConnection, getPythProgramKeyForCluster } from "@pythnetwork/client";
import { useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { useMemo } from "react";
import useSWRSubscription, { SWRSubscriptionOptions } from "swr/subscription";
import { Prices, TokenPrices } from "./types";
import { USD_DECIMALS, expandDecimals } from "@/components/MarketsList/utils";
import { toBN } from "gmsol";

export type PriceProvider = "pyth";

interface Request {
  key: "token-prices",
  provider: PriceProvider,
  feeds: PublicKey[],
}

export const usePriceFromFeeds = ({ provider = "pyth", feeds }: { provider?: PriceProvider, feeds: PublicKey[] }) => {
  const connection = useConnection();

  const request = useMemo<Request | null>(() => {
    return feeds.length > 0 ? {
      key: "token-prices",
      provider,
      feeds,
    } : null;
  }, [feeds, provider]);

  const { data } = useSWRSubscription(request, ({ feeds }, { next }: SWRSubscriptionOptions<Prices, Error>) => {
    const pubkey = getPythProgramKeyForCluster(DEFAULT_CLUSTER);
    const conn = new PythConnection(connection.connection, pubkey, undefined, feeds);
    conn.onPriceChange((product, price) => {
      const feedAddress = product.price_account;
      const priceValue = price.aggregate.priceComponent;
      const confidence = price.aggregate.confidenceComponent;
      const decimals = price.exponent;
      if (feedAddress && -decimals <= USD_DECIMALS) {
        const minPrice = expandDecimals(toBN(priceValue - confidence), USD_DECIMALS + decimals);
        const maxPrice = expandDecimals(toBN(priceValue + confidence), USD_DECIMALS + decimals);
        next(null, prices => {
          prices = prices ?? {};
          return {
            ...prices,
            [feedAddress]: {
              minPrice,
              maxPrice,
            } as TokenPrices,
          }
        });
      }
    });
    void conn.start();
    console.debug("pyth subscribed");
    return () => {
      console.debug("pyth unsubscribe");
      void conn.stop();
    }
  });

  const prices = useMemo(() => {
    return data ?? {}
  }, [data]);

  return prices;
};
