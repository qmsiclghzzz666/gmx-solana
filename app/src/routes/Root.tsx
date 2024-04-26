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

const Zoom = cssTransition({
  enter: "zoomIn",
  exit: "zoomOut",
  appendPosition: false,
  collapse: true,
  collapseDuration: 200,
});

export default function Root() {
  return (
    <>
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
    </>
  )
}
