import React from 'react';
import ReactDOM from 'react-dom/client';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import Root from './routes/Root';
import { AnchorContextProvider } from './components/AnchorContextProvider';

import { messages } from "./locales/en/messages";
import { I18nProvider } from "@lingui/react";
import { i18n } from "@lingui/core";

const router = createBrowserRouter([
  {
    path: "/",
    element: <Root />,
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
