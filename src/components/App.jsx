import { createAppState } from "@/api/appState";
import Journal from "@/components/Journal";
import styles from "@/styles/App";
import { DocumentEventListener } from "@solid-primitives/event-listener";
import { MetaProvider, Title } from "@solidjs/meta";
import { createHotkey } from "@tanstack/solid-hotkeys";
import { createSignal } from "solid-js";
import { onCloseRequested } from "@/api/event";
import { confirm } from "@tauri-apps/plugin-dialog";

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

  onCloseRequested(async ({ close }) => {
    if (
      !dirty() ||
      (await confirm(
        "Are you sure you want to exit? Unsaved changes will be lost!",
        { kind: "warning" },
      ))
    ) {
      close();
    }
  });

  return (
    <AppStateProvider>
      <MetaProvider>
        <Title>
          {dirty()
            ? `*${journalBasename() ?? "(untitled)"} \u2014 factbook`
            : journalBasename()
              ? `${journalBasename()} \u2014 factbook`
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
