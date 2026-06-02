import { invoke } from "@tauri-apps/api/core";
import { basename } from "@tauri-apps/api/path";
import { createResource } from "solid-js";

export default function useAppState() {
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

  return { state, journalPath, journalBasename, openJournal, closeJournal };
}
