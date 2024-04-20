import { GMSOLDeployment } from "gmsol";

declare global {
  interface Window {
    __GMSOL_DEPLOYMENT__?: GMSOLDeployment,
  }
}
