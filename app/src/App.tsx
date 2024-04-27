import { createBrowserRouter, Navigate, RouterProvider } from 'react-router-dom';
import Root from './routes/Root';

import { I18nProvider } from "@lingui/react";
import { i18n } from "@lingui/core";
import Earn from './routes/Earn';
import Dashboard from './routes/Dashboard';
import Exchange from './routes/Exchange';
import { StateProvider } from './contexts/state';
import { earnLoader } from './routes/loaders';
import { NativeTokenUtilsProvider } from './components/NativeTokenUtils';
import { useEffect } from 'react';
import { defaultLocale, dynamicActivate } from './utils/i18n';
import { LANGUAGE_LOCALSTORAGE_KEY } from './config/localStorage';
import { PendingStateProvider } from './contexts/pending';
import { SWRConfig } from 'swr';
import { AnchorStateProvider } from './contexts/anchor';

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
      },
      {
        path: "trade",
        element: <Exchange />,
      }
    ]
  }
]);

const swrConfig = {
  refreshInterval: 5000,
};

export function App() {
  useEffect(() => {
    const defaultLangugage = localStorage.getItem(LANGUAGE_LOCALSTORAGE_KEY) ?? defaultLocale;
    void dynamicActivate(defaultLangugage);
  }, []);

  return (
    <I18nProvider i18n={i18n}>
      <SWRConfig value={swrConfig}>
        <AnchorStateProvider>
          <PendingStateProvider>
            <StateProvider>
              <NativeTokenUtilsProvider>
                <RouterProvider router={router} />
              </NativeTokenUtilsProvider>
            </StateProvider>
          </PendingStateProvider>
        </AnchorStateProvider>
      </SWRConfig>
    </I18nProvider>
  );
}
