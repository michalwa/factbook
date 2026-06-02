import { createAppState } from "@/api/appState";
import Journal from "@/components/Journal";
import { Match, Switch } from "solid-js";
import styles from "@/styles/App";
import Start from "@/components/Start";
import { MetaProvider, Title } from "@solidjs/meta";
import { createHotkey } from "@tanstack/solid-hotkeys";

export default function App() {
  const {
    Provider: AppStateProvider,
    state,
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
        <Switch>
          <Match when={state() === "start"}>
            <Start />
          </Match>
          <Match when={state() === "journal"}>
            <Journal />
          </Match>
        </Switch>
      </div>
    </AppStateProvider>
  );
}
