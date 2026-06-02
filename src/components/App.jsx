import useAppState from "@/api/state";
import Journal from "@/components/Journal";
import { Match, Switch } from "solid-js";
import styles from "@/styles/App";
import Start from "./Start";
import { MetaProvider, Title } from "@solidjs/meta";

export default function App() {
  const { state, journalBasename, openJournal, closeJournal } = useAppState();

  return (
    <MetaProvider>
      <Show when={journalBasename()} fallback={<Title>factbook</Title>}>
        <Title>factbook &mdash; {journalBasename()}</Title>
      </Show>
      <div class={styles.app}>
        <Switch>
          <Match when={state() === "start"}>
            <Start onOpenJournal={openJournal} />
          </Match>
          <Match when={state() === "journal"}>
            <Journal onClose={closeJournal} />
          </Match>
        </Switch>
      </div>
    </MetaProvider>
  );
}
