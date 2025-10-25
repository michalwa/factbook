import { createContext, createSignal, useContext } from "solid-js";

const ViewContext = createContext();

/**
 * Non-null placeholder for the ID of the default view
 */
export const VIEW_ALL = "all";

export function Provider(props) {
  const selectedViewIdSignal = createSignal(VIEW_ALL);

  return (
    <ViewContext.Provider value={selectedViewIdSignal}>
      {props.children}
    </ViewContext.Provider>
  );
}

export function useViewContext() {
  return useContext(ViewContext);
}
