import { createCodeMirror } from "solid-codemirror";
import styles from "./CodeEditor.module.css";
import { EditorView } from "@codemirror/view";

export default function CodeEditor(props) {
  const { ref: editorRef, createExtension } = createCodeMirror({
    value: props.value,
  });

  createExtension(EditorView.lineWrapping);

  return <div ref={editorRef} class={`${styles.editor} ${props.class}`} />;
}
