import {
  createContext,
  createResource,
  createSignal,
  useContext,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";

const ViewContext = createContext();

export const VIEW_ALL = "all";
export const VIEW_NEW = "new";

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

  function beginCreateView() {
    if (!views()?.some((view) => view.id === VIEW_NEW))
      mutateViews((views) => [...views, { id: VIEW_NEW }]);

    setSelectedViewId(VIEW_NEW);
  }

  async function commitCreateView() {
    const view = views()?.find((view) => view.id === VIEW_NEW);
    if (!view) return;
    view.id = await invoke("create_view", view);
  }

  function setSelectedViewIdDiscardingNew(id) {
    setSelectedViewId(id);
    if (id !== VIEW_NEW)
      mutateViews((views) => views.filter((view) => view.id !== VIEW_NEW));
  }

  const context = {
    views: viewsWithAll,
    selectedViewId,
    setSelectedViewId: setSelectedViewIdDiscardingNew,
    beginCreateView,
    commitCreateView,
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
