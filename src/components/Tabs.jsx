import { createContext, createSignal, createUniqueId } from "solid-js";
import styles from "@/styles/Tabs";

export const TabsContext = createContext();

export default function Tabs(props) {
  const name = props.name ?? createUniqueId();
  const [activeId, setActiveId] = props.activeId ?? createSignal(0);

  return (
    <TabsContext.Provider value={{ name, activeId, setActiveId }}>
      <div class={styles.tabs}>{props.children}</div>
    </TabsContext.Provider>
  );
}
