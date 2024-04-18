import React from 'react';
import ReactDOM from 'react-dom/client';
import { createBrowserRouter, Navigate, RouterProvider } from 'react-router-dom';
import Root from './routes/Root';
import { AnchorContextProvider } from './components/AnchorContextProvider';

import { messages } from "./locales/en/messages";
import { I18nProvider } from "@lingui/react";
import { i18n } from "@lingui/core";
import Stake from './routes/Stake';
import Dashboard from './routes/Dashboard';

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
      }
    ]
  }
]);

i18n.load("en", messages);
i18n.activate("en");

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <AnchorContextProvider>
      <I18nProvider i18n={i18n}>
        <RouterProvider router={router} />
      </I18nProvider>
    </AnchorContextProvider>
  </React.StrictMode>,
)
