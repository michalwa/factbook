import { invoke } from "@tauri-apps/api/core";
import { createResource } from "solid-js";

export default function useAppState() {
  const [state, { refetch: refetchState }] = createResource(() =>
    invoke("get_state"),
  );

  const openJournal = async (path) => {
    await invoke("open_journal", { path });
    await refetchState();
  };

  return { state, openJournal };
}
