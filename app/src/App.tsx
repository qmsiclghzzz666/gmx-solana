import { createBrowserRouter, Navigate, RouterProvider } from 'react-router-dom';
import Root from './routes/Root';

import { I18nProvider } from "@lingui/react";
import { i18n } from "@lingui/core";
import Earn from './routes/Earn';
import Dashboard from './routes/Dashboard';
import Exchange from './routes/Exchange';
import { earnLoader } from './routes/loaders';
import { useEffect } from 'react';
import { defaultLocale, dynamicActivate } from './utils/i18n';
import { LANGUAGE_LOCALSTORAGE_KEY } from './config/localStorage';
import { SWRConfig } from 'swr';
import { AnchorStateProvider } from './contexts/anchor';
import { Governance } from './routes/Governance';

const router = createBrowserRouter([
  {
    path: "/",
    element: <Root />,
    children: [
      {
        index: true,
        element: <Navigate to="/trade" />,
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
  },
  {
    path: "/governance",
    element: <Governance />
  }
]);

const swrConfig = {
  refreshInterval: 5000,
};

export function App() {
  useEffect(() => {
    const defaultLanguage = localStorage.getItem(LANGUAGE_LOCALSTORAGE_KEY) ?? defaultLocale;
    void dynamicActivate(defaultLanguage);
  }, []);

  return (
    <I18nProvider i18n={i18n}>
      <SWRConfig value={swrConfig}>
        <AnchorStateProvider>
          <RouterProvider router={router} />
        </AnchorStateProvider>
      </SWRConfig>
    </I18nProvider>
  );
}
