import { invoke } from "@tauri-apps/api/core";
import { createResource } from "solid-js";
import { useAppState } from "@/api/appState";

export const defaultView = {
  id: null,
  name: "All",
};

export function useViews() {
  const { setDirty } = useAppState();

  const [createdViews, { refetch: refetchViews, mutate: mutateViews }] =
    createResource(() => invoke("get_views"));
  const views = () => [defaultView, ...(createdViews() ?? [])];

  const getView = (id) => views().find((view) => view.id === id);

  const setViewName = async (id, name) => {
    setDirty(true);
    await invoke("set_view_name", { id, name });
    mutateViews((views) =>
      views.map((view) => (view.id === id ? { ...view, name } : view)),
    );
  };

  const setViewDefinition = async (id, definition) => {
    setDirty(true);
    await invoke("set_view_definition", { id, definition });
    await refetchViews(); // update entry counts
  };

  const createView = async () => {
    setDirty(true);
    const id = await invoke("create_view");
    await invoke("set_view_name", { id, name: "(untitled)" });
    await invoke("set_view_definition", { id, definition: "any" });
    await refetchViews();
    return id;
  };

  const removeView = async (id) => {
    setDirty(true);
    await invoke("remove_view", { id });
    mutateViews((views) => views.filter((view) => view.id !== id));
  };

  return {
    views,
    refetchViews,
    getView,
    setViewName,
    setViewDefinition,
    createView,
    removeView,
  };
}
