import { useContext } from "solid-js";
import styles from "@/styles/Tab";
import { TabsContext } from "@/components/Tabs";

export default function Tab(props) {
  const { name, currentId, setCurrentId } = useContext(TabsContext);

  return (
    <label class={styles.tab}>
      <input
        class={styles.radio}
        type="radio"
        name={name}
        value={props.id}
        checked={props.id === currentId()}
        onClick={() => setCurrentId(props.id)}
      />
      <span class={styles.title}>
        {/* In case of empty names, put a zero-width space to maintain height */}
        {props.title || "\u200b"}
        <span class={styles.controls}>{props.controls}</span>
      </span>
      {props.children}
    </label>
  );
}
