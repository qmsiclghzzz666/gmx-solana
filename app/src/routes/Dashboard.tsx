import PageTitle from "@/components/PageTitle/PageTitle";
import { Trans, t } from "@lingui/macro";
import { MarketsList } from "@/components/MarketsList/MarketsList";

import "./Dashboard.css";

export default function Dashboard() {
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
    </div>
  )
}
