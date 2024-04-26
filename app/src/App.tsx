import { createBrowserRouter, Navigate, RouterProvider } from 'react-router-dom';
import Root from './routes/Root';

import { I18nProvider } from "@lingui/react";
import { i18n } from "@lingui/core";
import Earn from './routes/Earn';
import Dashboard from './routes/Dashboard';
import Exchange from './routes/Exchange';
import { OnChainProvider } from './onchain';
import { StateProvider } from './contexts/state';
import { earnLoader } from './routes/loaders';
import { action as gmAction } from "@/components/GmSwap/GmSwapBox/action";
import { NativeTokenUtilsProvider } from './components/NativeTokenUtils';

const router = createBrowserRouter([
  {
    path: "/",
    element: <Root />,
    children: [
      {
        index: true,
        element: <Navigate to="/dashboard" />,
      },
      {
        path: "dashboard",
        element: <Dashboard />,
      },
      {
        path: "earn",
        element: <Earn />,
        loader: earnLoader,
        action: gmAction,
      },
      {
        path: "trade",
        element: <Exchange />,
      }
    ]
  }
]);

i18n.load({ "en": {} });
i18n.activate("en");

export function App() {
  return (
    <I18nProvider i18n={i18n}>
      <OnChainProvider>
        <StateProvider>
          <NativeTokenUtilsProvider>
            <RouterProvider router={router} />
          </NativeTokenUtilsProvider>
        </StateProvider>
      </OnChainProvider>
    </I18nProvider>
  );
}
