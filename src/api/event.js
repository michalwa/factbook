import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { onCleanup, onMount } from "solid-js";

/**
 * @param {(params: { close: () => void }) => void} callback
 */
export function onCloseRequested(callback) {
  onMount(() => {
    const window = getCurrentWebviewWindow();
    const unlisten = window.listen("tauri://close-requested", () =>
      callback({
        close() {
          window.destroy();
        },
      }),
    );

    onCleanup(async () => (await unlisten)?.());
  });
}
