import styles from "@/styles/FileInput";
import { createEffect, createSignal, on } from "solid-js";
import { open as openNativeFileDialog } from "@tauri-apps/plugin-dialog";

export default function FileInput(props) {
  const [filePath, setFilePath] = createSignal();
  createEffect(on(filePath, (path) => props.onChange?.(path)));

  const open = async () => {
    const filePath = await openNativeFileDialog({
      filters: props.filters,
    });

    if (filePath) setFilePath(filePath);
  };

  return (
    <button
      class={`${styles.button} ${filePath() && styles.selected}`}
      onClick={open}
    >
      {filePath() ?? "Choose file..."}
    </button>
  );
}
