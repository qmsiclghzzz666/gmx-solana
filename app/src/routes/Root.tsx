import "@/styles/Font.css";
import "@/styles/Input.css";
import "@/styles/Shared.scss";
import "@/styles/GlpSwap.css";
import "@/styles/AddressDropdown.scss";
import "./Root.scss";
import { Header } from "@/components/Header/Header";
import { Outlet } from "react-router-dom";
import Footer from "@/components/Footer/Footer";

export default function Root() {
  return (
    <>
      <div className="App">
        <div className="App-content">
          <Header />
        </div>
        <Outlet />
        <Footer />
      </div>
    </>
  )
}
