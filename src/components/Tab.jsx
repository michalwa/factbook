import { useContext } from "solid-js";
import styles from "@/styles/Tab";
import { TabsContext } from "@/components/Tabs";

export default function Tab(props) {
  const { name, activeId, setActiveId } = useContext(TabsContext);

  return (
    <label class={styles.tab}>
      <input
        class={styles.radio}
        type="radio"
        name={name}
        value={props.id}
        checked={props.id === activeId()}
        onClick={() => setActiveId(props.id)}
      />
      <span class={styles.title}>{props.title}</span>
      {props.children}
    </label>
  );
}
