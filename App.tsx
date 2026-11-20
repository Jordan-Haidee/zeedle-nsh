import { mount } from "nsh";
import { Window } from "nsh/desktop";
import { App } from "./AppView";

mount(() => (
  <Window
    id="zeedle-main"
    title="Zeedle"
    titleBar={{ style: "hidden", height: 40 }}
    defaultSize={{ width: 900, height: 510 }}
    minSize={{ width: 850, height: 500 }}
  >
    {App}
  </Window>
));
