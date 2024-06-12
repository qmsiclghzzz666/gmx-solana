import "@/styles/Font.css";
import "@/styles/Input.css";
import "@/styles/Shared.scss";
import "@/styles/GlpSwap.css";
import "@/styles/AddressDropdown.scss";

import 'react-toastify/dist/ReactToastify.css';
import "./Root.scss";

import { Header } from "@/components/Header/Header";
import { Outlet } from "react-router-dom";
import Footer from "@/components/Footer/Footer";
import { ToastContainer, cssTransition } from "react-toastify";
import { TOAST_AUTO_CLOSE_TIME } from '@/config/ui';
import { NativeTokenUtilsProvider } from '@/components/NativeTokenUtils';
import { SharedStatesProvider } from '@/contexts/shared';
import { PendingStateProvider } from '@/contexts/pending';

const Zoom = cssTransition({
  enter: "zoomIn",
  exit: "zoomOut",
  appendPosition: false,
  collapse: true,
  collapseDuration: 200,
});

export default function Root() {
  return (
    <PendingStateProvider>
      <SharedStatesProvider>
        <NativeTokenUtilsProvider>
          <div className="App">
            <div className="App-content">
              <Header />
            </div>
            <Outlet />
            <Footer />
            <ToastContainer
              limit={1}
              theme="dark"
              transition={Zoom}
              position="bottom-right"
              autoClose={TOAST_AUTO_CLOSE_TIME}
              hideProgressBar={true}
              newestOnTop={false}
              closeOnClick={false}
              draggable={false}
              pauseOnHover
            />
          </div>
        </NativeTokenUtilsProvider>
      </SharedStatesProvider>
    </PendingStateProvider>
  )
}
