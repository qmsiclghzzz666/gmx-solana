import { GMSOLDeployment } from "./config/deployment";

declare global {
  interface Window {
    __GMSOL_DEPLOYMENT__?: GMSOLDeployment,
  }
}