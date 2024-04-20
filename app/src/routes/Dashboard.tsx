import PageTitle from "@/components/PageTitle/PageTitle";
import { Trans, t } from "@lingui/macro";
import Footer from "@/components/Footer/Footer";
import { MarketsList } from "@/components/MarketsList/MarketsList";

import "./Dashboard.css";

// interface Market {
//   long?: string,
//   short?: string,
//   pubkey?: PublicKey,
// }

export default function Dashboard() {
  // const dataStore = useDataStore();
  // const [market, setMarket] = useState<Market | null>(null);

  // useEffect(() => {
  //   const fetchMarket = async () => {
  //     const deployment = getGMSOLDeployment();

  //     if (deployment) {
  //       const [address] = findMarketPDA(deployment.store, deployment.markets[0].market_token);
  //       const data = await dataStore?.account.market.fetch(address);
  //       setMarket({
  //         long: data?.pools.pools[0].longTokenAmount.toString(),
  //         short: data?.pools.pools[0].shortTokenAmount.toString(),
  //         pubkey: dataStore?.provider.publicKey,
  //       });
  //     }
  //   };

  //   void fetchMarket();
  // }, [dataStore]);

  return (
    <div className="default-container page-layout DashboardV2">
      <PageTitle
        title={t`Stats`}
        isTop
        subtitle={
          <div>
            <Trans>
              Total Stats of GMSOL.
            </Trans>
          </div>
        }
      />
      <div className="DashboardV2-content">
        <div className="DashboardV2-token-cards">
          <MarketsList />
        </div>
      </div>
      <Footer />
    </div>
  )
}
