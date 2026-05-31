import { invoke } from "@tauri-apps/api/core";
import { createResource } from "solid-js";

export const defaultView = {
  id: null,
  name: "All",
};

export function useViews() {
  const [createdViews, { refetch: refetchViews }] = createResource(() =>
    invoke("get_views"),
  );
  const views = () => [defaultView, ...(createdViews() ?? [])];

  const getView = (id) => views().find((view) => view.id === id);

  const setViewDefinition = async (id, definition) => {
    await invoke("set_view_definition", { id, definition });
    refetchViews();
  };

  return { views, getView, setViewDefinition };
}
