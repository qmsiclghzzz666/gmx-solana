import { HermesClient } from "@pythnetwork/hermes-client";
import { useMemo, useRef } from "react";
import useSWRSubscription, { SWRSubscriptionOptions } from "swr/subscription";
import { Prices, TokenPrices } from "./types";
import { expandDecimals } from "@/utils/number";
import { USD_DECIMALS } from "@/config/constants";
import { BN } from "@coral-xyz/anchor";

export type PriceProvider = "pyth";

const HERMES_ENDPOINT = "https://hermes.pyth.network";

interface Request {
  key: "token-prices",
  provider: PriceProvider,
  feeds: string[],
}

interface PythData {
  parsed: PriceUpdate[]
}

interface PriceUpdate {
  id: string,
  price: {
    price: string,
    conf: string,
    expo: number,
  }
}

export const usePriceFromFeeds = ({ provider = "pyth", feeds }: { provider?: PriceProvider, feeds: string[] }) => {
  const eventSource = useRef<EventSource | null>(null);

  const request = useMemo<Request | null>(() => {
    return feeds.length > 0 ? {
      key: "token-prices",
      provider,
      feeds,
    } : null;
  }, [feeds, provider]);

  const { data } = useSWRSubscription(request, ({ feeds }, { next }: SWRSubscriptionOptions<Prices, Error>) => {
    // const conn = new PythConnection(connection.connection, pubkey, undefined, feeds);
    const conn = new HermesClient(HERMES_ENDPOINT);
    const subscribe = async () => {
      eventSource.current = await conn.getPriceUpdatesStream(feeds) as EventSource;
      eventSource.current.onmessage = (event: MessageEvent<string>) => {
        const data = JSON.parse(event.data) as PythData;
        for (const update of data.parsed) {
          const feedId = `0x${update.id}`;
          const midPrice = new BN(update.price.price);
          const conf = new BN(update.price.conf);
          const expo = update.price.expo;
          const minPrice = expandDecimals((midPrice.sub(conf)), USD_DECIMALS + expo);
          const maxPrice = expandDecimals((midPrice.add(conf)), USD_DECIMALS + expo);
          next(null, prices => {
            return {
              ...prices,
              [feedId]: {
                minPrice,
                maxPrice
              } as TokenPrices,
            }
          });
        }
      }
    };
    void subscribe();
    console.debug("pyth subscribed");
    return () => {
      console.debug("pyth unsubscribe");
      if (eventSource.current) {
        eventSource.current.close();
      }
    }
  });

  const prices = useMemo(() => {
    return data ?? {}
  }, [data]);

  return prices;
};
