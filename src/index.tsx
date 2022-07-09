/* @refresh reload */
import { render } from "solid-js/web";
import { HopeProvider } from "@hope-ui/solid";

import Saxo from "./Saxo";

const App = (
  <HopeProvider>
    <Saxo />
  </HopeProvider>
);

render(() => App, document.getElementById("root") as HTMLElement);
