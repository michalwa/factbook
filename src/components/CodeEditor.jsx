import { createCodeMirror } from "solid-codemirror";
import styles from "@/styles/CodeEditor";
import { EditorView } from "@codemirror/view";
import { onCleanup } from "solid-js";

export default function CodeEditor(props) {
  const { ref: editorRef, createExtension } = createCodeMirror({
    value: props.value,
  });

  createExtension(EditorView.lineWrapping);

  onCleanup(() => console.log("editor dispose"));

  return <div ref={editorRef} class={`${styles.editor} ${props.class}`} />;
}
