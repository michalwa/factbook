import {
  createContext,
  createResource,
  createSignal,
  useContext,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { debounce } from "@solid-primitives/scheduled";
import { mapWhere } from "./utils";

const ViewContext = createContext();

export const VIEW_ALL = "all";

export function Provider(props) {
  const [views, { mutate: mutateViews }] = createResource(() =>
    invoke("get_views"),
  );
  const [selectedViewId, setSelectedViewId] = createSignal(VIEW_ALL);

  const viewsWithAll = () => [
    {
      id: VIEW_ALL,
      name: "All",
      entryCount: 0,
    },
    ...(views() || []),
  ];

  const view = () => views()?.find((view) => view.id === selectedViewId());

  async function createView() {
    const id = await invoke("create_view");
    mutateViews((views) => [...views, { id, entryCount: 0 }]);
    setSelectedViewId(id);
  }

  const setViewNameDebounced = debounce(
    (id, name) => invoke("set_view_name", { view: id, name }),
    200,
  );

  async function setViewName(name) {
    const viewId = selectedViewId();
    if (viewId === VIEW_ALL) return;

    mutateViews(
      mapWhere(
        (view) => view.id === viewId,
        (view) => ({ ...view, name }),
      ),
    );

    setViewNameDebounced(viewId, name);
  }

  const context = {
    views: viewsWithAll,
    view,
    selectedViewId,
    setSelectedViewId,
    createView,
    setViewName,
  };

  return (
    <ViewContext.Provider value={context}>
      {props.children}
    </ViewContext.Provider>
  );
}

export function useViewContext() {
  return useContext(ViewContext);
}
