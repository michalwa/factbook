import Resizable from "@corvu/resizable";
import "@fontsource-variable/inter/wght.css";
import { makePersisted } from "@solid-primitives/storage";
import Sidebar from "./Sidebar";
import "./App.css";
import EntryList from "./EntryList";
import * as ViewContext from "./ViewContext";
import { createSignal } from "solid-js";

export default function App() {
  const [sizes, setSizes] = makePersisted(createSignal([]), {
    name: "panel-sizes",
  });

  return (
    <div id="app">
      <ViewContext.Provider>
        <Resizable
          orientation="horizontal"
          sizes={sizes()}
          onSizesChange={setSizes}
        >
          <Resizable.Panel
            collapsible
            initialSize={0.3}
            minSize={0.2}
            maxSize={0.5}
          >
            <Sidebar />
          </Resizable.Panel>
          <Resizable.Handle />
          <Resizable.Panel initialSize={0.7}>
            <EntryList />
          </Resizable.Panel>
        </Resizable>
      </ViewContext.Provider>
    </div>
  );
}
