import { invoke } from "@tauri-apps/api/core";
import { createResource } from "solid-js";

export const defaultView = {
  id: null,
  name: "All",
};

export function useViews() {
  const [createdViews] = createResource(() => invoke("get_views"));
  const views = () => [defaultView, ...(createdViews() ?? [])];

  return { views };
}
