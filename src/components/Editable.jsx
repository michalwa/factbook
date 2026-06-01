import { ContentEditable } from "@bigmistqke/solid-contenteditable";
import { createSignal } from "solid-js";
import styles from "@/styles/Editable";

export default function createEditable(props) {
  const [editing, setEditing] = createSignal(false);
  const [editedValue, setEditedValue] = createSignal(props.value());

  let ref;

  const edit = () => {
    setEditing(true);
    ref.focus();

    const range = document.createRange();
    range.selectNodeContents(ref);

    const selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
  };

  const save = () => {
    if (editing()) {
      setEditedValue(editedValue().trim());
      props.onChange?.(editedValue());
      setEditing(false);
    }
    ref.blur();
  };

  const reset = () => {
    if (editing()) {
      setEditedValue(props.value);
      setEditing(false);
    }
    ref.blur();
  };

  const onValueChange = (value) => {
    setEditedValue(value);
    queueMicrotask(() => scrollToCursor());
  };

  const onBlur = () => {
    reset();
    ref.scrollLeft = 0;
  };

  const onKeyDown = (event) => {
    if (event.key === "Enter") save();
    if (event.key === "Escape") reset();
    if (
      [
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "ArrowDown",
        "Home",
        "End",
      ].includes(event.key)
    )
      scrollToCursor();
  };

  const scrollToCursor = () => {
    const selection = window.getSelection();
    if (!selection.rangeCount) return;

    const range = selection.getRangeAt(0);
    const cursorRect = range.getBoundingClientRect();
    const containerRect = ref.getBoundingClientRect();

    if (cursorRect.right > containerRect.right) {
      ref.scrollLeft += cursorRect.right - containerRect.right;
    } else if (cursorRect.left < containerRect.left) {
      ref.scrollLeft += cursorRect.left - containerRect.left;
    }
  };

  const Editable = (props) => (
    <ContentEditable
      ref={ref}
      class={`${styles.editable} ${props.class}`}
      editable={editing()}
      textContent={editedValue()}
      onTextContent={onValueChange}
      onBlur={onBlur}
      onKeyDown={onKeyDown}
      onKeyUp={scrollToCursor}
      singleline
      tabindex="-1"
    />
  );

  return {
    Editable,
    edit,
    save,
    reset,
    editing,
  };
}
