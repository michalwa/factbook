import Resizable from "@corvu/resizable";
import "@fontsource-variable/inter/wght.css";
import Sidebar from "./Sidebar";
import "./App.css";
import EntryList from "./EntryList";

export default function App() {
  return (
    <div id="app">
      <Resizable orientation="horizontal">
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
    </div>
  );
}
