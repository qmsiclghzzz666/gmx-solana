import "styles/Font.css";
import "styles/Input.css";
import "styles/Shared.scss";
import "./Root.scss";
import { Header } from "components/Header/Header";

export default function Root() {
  return (
    <>
      <div className="App">
        <div className="App-content">
          <Header />
        </div>
        <div style={{ height: '200px' }}>
        </div>
      </div>
    </>
  )
}
