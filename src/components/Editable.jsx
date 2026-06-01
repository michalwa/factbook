import { ContentEditable } from "@bigmistqke/solid-contenteditable";
import { createSignal } from "solid-js";
import styles from "@/styles/Editable";

/**
 * NOTE: The returned component does not handle discarding the edit on blur to
 * allow submitting via buttons. Use `use:clickOutside` on an outer element
 * and call `reset` to implement this behavior manually.
 */
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

  // TODO: Consider reimplementing this component as a seamless <input> to maybe
  // reduce the need for some of the manual handling and scrolling
  const onKeyDown = (event) => {
    if (event.key === "Enter") save();
    else if (event.key === "Escape") reset();
    else scrollToCursor();
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
      class={`${styles.editable} ${props.class} ${editing() && props.editingClass}`}
      editable={editing()}
      textContent={editedValue()}
      onTextContent={onValueChange}
      onBlur={() => (ref.scrollLeft = 0)}
      onKeyDown={onKeyDown}
      onKeyUp={scrollToCursor}
      onClick={(e) => e.preventDefault()}
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
