import { createResource } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export function useEntries(viewId) {
  const [entries] = createResource(
    () => ({ viewId: viewId() }), // construct an object to treat `null` as a valid value
    ({ viewId }) => invoke("get_entries", { view: viewId }),
  );

  return { entries };
}
