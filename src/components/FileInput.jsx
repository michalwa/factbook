import styles from "@/styles/FileInput";
import { createEffect, createSignal, on } from "solid-js";
import { save as saveFileDialog } from "@tauri-apps/plugin-dialog";

export default function FileInput(props) {
  const [filePath, setFilePath] = createSignal();
  createEffect(on(filePath, (path) => props.onChange?.(path)));

  const open = async () => {
    // FIXME: Is there any way to get the best of both worlds? i.e. ability to
    // create the file if needed but without the overwrite warning if opening
    // an existing file? Do we need two buttons?
    const filePath = await saveFileDialog({
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
