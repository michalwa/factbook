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
  const [state, { refetch: refetchState }] = createResource(() =>
    invoke("get_state"),
  );

  const [journalPath] = createResource(state, () => invoke("get_journal_path"));
  const [journalBasename] = createResource(
    () => ({ path: journalPath() }),
    ({ path }) => path && basename(path),
  );

  const openJournal = async (path) => {
    await invoke("open_journal", { path });
    await refetchState();
  };

  const closeJournal = async (path) => {
    await invoke("close_journal", { path });
    await refetchState();
  };

  const [dirty, setDirty] = createSignal(false);

  const saveJournal = async () => {
    await invoke("save_journal");
    setDirty(false);
  };

  const context = {
    state,
    journalPath,
    journalBasename,
    openJournal,
    closeJournal,
    dirty,
    setDirty,
    saveJournal,
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
