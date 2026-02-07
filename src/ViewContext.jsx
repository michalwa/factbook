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
  const [viewJustCreated, setViewJustCreated] = createSignal(false, {
    equals: false,
  });

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
    setViewJustCreated(true);
  }

  const setViewNameDebounced = debounce(
    (id, name) => invoke("set_view_name", { id, name }),
    200,
  );

  function setViewName(name) {
    const id = selectedViewId();
    if (id === VIEW_ALL) return;

    mutateViews(
      mapWhere(
        (view) => view.id === id,
        (view) => ({ ...view, name }),
      ),
    );

    setViewNameDebounced(id, name);
  }

  const setViewDefinitionDebounced = debounce(
    (id, definition) => invoke("set_view_definition", { id, definition }),
    200,
  );

  function setViewDefinition(definition) {
    const id = selectedViewId();
    if (id === VIEW_ALL) return;

    mutateViews(
      mapWhere(
        (view) => view.id === id,
        (view) => ({ ...view, definition }),
      ),
    );

    setViewDefinitionDebounced(id, definition);
  }

  async function removeView() {
    const id = selectedViewId();
    if (id === VIEW_ALL) return;

    await invoke("remove_view", { id });
    mutateViews((views) => views.filter((view) => view.id !== id));
  }

  const context = {
    views: viewsWithAll,
    view,
    selectedViewId,
    setSelectedViewId,
    createView,
    viewJustCreated,
    setViewName,
    setViewDefinition,
    removeView,
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
