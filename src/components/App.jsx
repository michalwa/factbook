import useAppState from "@/api/state";
import Journal from "@/components/Journal";
import { Match, Switch } from "solid-js";
import styles from "@/styles/App";
import Start from "./Start";

export default function App() {
  const { state, openJournal, closeJournal } = useAppState();

  return (
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
  );
}
