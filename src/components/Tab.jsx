import { useContext } from "solid-js";
import styles from "@/styles/Tab";
import { TabsContext } from "@/components/Tabs";
import createEditable from "@/components/Editable";
import { clickOutside } from "@/utils";

export default function Tab(props) {
  const { name, currentId, setCurrentId } = useContext(TabsContext);

  const {
    Editable: EditableTitle,
    edit: editTitle,
    save: saveTitle,
    reset: resetTitle,
    editing: editingTitle,
  } = createEditable({
    value: () => props.title,
    onChange: (title) => props.onTitleChange?.(title),
  });

  const childrenContext = {
    editTitle,
    saveTitle,
    resetTitle,
    editingTitle,
  };

  return (
    <label class={styles.tab} use:clickOutside={resetTitle}>
      <input
        class={styles.radio}
        type="radio"
        name={name}
        value={props.id}
        checked={props.id === currentId()}
        onClick={() => setCurrentId(props.id)}
      />
      <div class={styles.titleContainer}>
        <EditableTitle
          class={styles.title}
          editingClass={styles.titleEditing}
        />
        <div class={styles.controls}>{props.controls(childrenContext)}</div>
      </div>
      {props.children(childrenContext)}
    </label>
  );
}
