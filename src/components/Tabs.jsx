import { createContext, createSignal, createUniqueId } from "solid-js";
import styles from "@/styles/Tabs";

export const TabsContext = createContext();

export default function Tabs(props) {
  const name = createUniqueId();
  const currentId = () => props.currentId;
  const setCurrentId = (id) => props.onCurrentChange?.(id);

  return (
    <TabsContext.Provider value={{ name, currentId, setCurrentId }}>
      <div class={styles.tabs}>{props.children}</div>
    </TabsContext.Provider>
  );
}
