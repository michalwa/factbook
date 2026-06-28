import { invoke } from "@tauri-apps/api/core";
import { basename } from "@tauri-apps/api/path";
import {
  createContext,
  createResource,
  createSignal,
  useContext,
} from "solid-js";

const AppStateContext = createContext();

export function createAppState() {
  const [journalPath, { refetch: refetchJournalPath }] = createResource(() =>
    invoke("get_journal_path"),
  );
  const [journalBasename] = createResource(
    () => ({ path: journalPath() }),
    ({ path }) => path && basename(path),
  );

  const [dirty, setDirty] = createSignal(false);
  const [lastSaved, setLastSaved] = createSignal();

  const createJournal = () => invoke("create_journal");
  const openJournal = () => invoke("open_journal");
  const saveJournal = async () => {
    const { success } = await invoke("save_journal");
    if (success) {
      setDirty(false);
      setLastSaved(new Date());
      refetchJournalPath();
    }
  };

  const openDefaultJournal = () => invoke("open_default_journal");

  const context = {
    journalPath,
    journalBasename,
    createJournal,
    openJournal,
    saveJournal,
    openDefaultJournal,
    dirty,
    setDirty,
    lastSaved,
  };

  const Provider = (props) => (
    <AppStateContext.Provider value={context}>
      {props.children}
    </AppStateContext.Provider>
  );

  return { ...context, Provider };
}

export function useAppState() {
  return useContext(AppStateContext);
}
