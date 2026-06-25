import { createAppState } from "@/api/appState";
import Journal from "@/components/Journal";
import styles from "@/styles/App";
import { DocumentEventListener } from "@solid-primitives/event-listener";
import { MetaProvider, Title } from "@solidjs/meta";
import { createHotkey } from "@tanstack/solid-hotkeys";
import { createSignal } from "solid-js";

export default function App() {
  const {
    Provider: AppStateProvider,
    journalBasename,
    dirty,
    saveJournal,
  } = createAppState();

  createHotkey("Mod+S", () => saveJournal());

  const [modKeyPressed, setModKeyPressed] = createSignal(false);

  const updateModKeyState = (event) =>
    setModKeyPressed(event.ctrlKey || event.metaKey);

  return (
    <AppStateProvider>
      <MetaProvider>
        <Title>
          {journalBasename()
            ? `${dirty() ? "*" : ""}${journalBasename()} \u2014 factbook`
            : "factbook"}
        </Title>
      </MetaProvider>
      <div class={`${styles.app} ${modKeyPressed() ? "mod-key-pressed" : ""}`}>
        <Journal />
      </div>
      <DocumentEventListener
        onKeyDown={updateModKeyState}
        onKeyUp={updateModKeyState}
        onMouseEnter={updateModKeyState}
        onMouseLeave={() => setModKeyPressed(false)}
      />
    </AppStateProvider>
  );
}
