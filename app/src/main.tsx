import React from 'react';
import ReactDOM from 'react-dom/client';
import { createBrowserRouter, Navigate, RouterProvider } from 'react-router-dom';
import Root from './routes/Root';

import { messages } from "./locales/en/messages";
import { I18nProvider } from "@lingui/react";
import { i18n } from "@lingui/core";
import Stake from './routes/Stake';
import Dashboard from './routes/Dashboard';
import Exchange from './routes/Exchange';
import { OnChainProvider } from './onchain';
import { StateProvider } from './contexts/state';

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
        element: <Stake />,
      },
      {
        path: "trade",
        element: <Exchange />,
      }
    ]
  }
]);

i18n.load("en", messages);
i18n.activate("en");

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <I18nProvider i18n={i18n}>
      <OnChainProvider>
        <StateProvider>
          <RouterProvider router={router} />
        </StateProvider>
      </OnChainProvider>
    </I18nProvider>
  </React.StrictMode>,
);
