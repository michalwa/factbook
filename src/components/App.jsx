import { createAppState } from "@/api/appState";
import Journal from "@/components/Journal";
import styles from "@/styles/App";
import { MetaProvider, Title } from "@solidjs/meta";
import { createHotkey } from "@tanstack/solid-hotkeys";

export default function App() {
  const {
    Provider: AppStateProvider,
    journalBasename,
    dirty,
    saveJournal,
  } = createAppState();

  createHotkey("Mod+S", () => saveJournal());

  return (
    <AppStateProvider>
      <MetaProvider>
        <Title>
          {journalBasename()
            ? `${dirty() ? "*" : ""}${journalBasename()} \u2014 factbook`
            : "factbook"}
        </Title>
      </MetaProvider>
      <div class={styles.app}>
        <Journal />
      </div>
    </AppStateProvider>
  );
}
