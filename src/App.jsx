import Resizable from "@corvu/resizable";
import "@fontsource-variable/inter/wght.css";
import { makePersisted } from "@solid-primitives/storage";
import Sidebar from "./Sidebar";
import "./App.css";
import EntryList from "./EntryList";
import * as ViewContext from "./ViewContext";
import { createStore } from "solid-js/store";
import ViewEditor from "./ViewEditor";
import CollapsiblePanel from "./CollapsiblePanel";
import { PanelBottomOpenIcon, PanelLeftOpenIcon } from "lucide-solid";

export default function App() {
  const [sizes, setSizes] = makePersisted(
    createStore({
      root: [],
      main: [],
    }),
    { name: "panel-sizes" },
  );

  return (
    <div id="app">
      <ViewContext.Provider>
        <Resizable
          orientation="horizontal"
          sizes={sizes.root}
          onSizesChange={(sizes) => setSizes("root", sizes)}
        >
          <CollapsiblePanel
            initialSize={0.3}
            minSize={0.2}
            maxSize={0.5}
            expandIcon={<PanelLeftOpenIcon />}
            expandButtonHorizontalAlign="left"
            expandButtonVerticalAlign="top"
          >
            <Sidebar />
          </CollapsiblePanel>
          <Resizable.Handle />
          <Resizable.Panel initialSize={0.7}>
            <Resizable
              orientation="vertical"
              sizes={sizes.main}
              onSizesChange={(sizes) => setSizes("main", sizes)}
            >
              <Resizable.Panel initialSize={0.6}>
                <EntryList />
              </Resizable.Panel>
              <Resizable.Handle />
              <CollapsiblePanel
                initialSize={0.4}
                minSize={0.2}
                maxSize={0.8}
                expandIcon={<PanelBottomOpenIcon />}
                expandButtonHorizontalAlign="right"
                expandButtonVerticalAlign="bottom"
              >
                <ViewEditor />
              </CollapsiblePanel>
            </Resizable>
          </Resizable.Panel>
        </Resizable>
      </ViewContext.Provider>
    </div>
  );
}
