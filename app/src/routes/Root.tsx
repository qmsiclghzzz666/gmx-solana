import "styles/Font.css";
import "styles/Input.css";
import "styles/Shared.scss";
import "./Root.scss";
import { Header } from "components/Header/Header";
import { Outlet } from "react-router-dom";

export default function Root() {
  return (
    <>
      <div className="App">
        <div className="App-content">
          <Header />
        </div>
        <Outlet />
        <div style={{ height: '200px' }}>
        </div>
      </div>
    </>
  )
}
