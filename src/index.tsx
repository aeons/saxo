/* @refresh reload */
import { render } from "solid-js/web";
import { HopeProvider, HopeThemeConfig } from "@hope-ui/solid";

import Saxo from "./Saxo";
import "./index.css";

const config: HopeThemeConfig = {
  initialColorMode: "dark",
};

const app = () => (
  <HopeProvider config={config}>
    <Saxo />
  </HopeProvider>
);

render(app, document.getElementById("root") as HTMLElement);
