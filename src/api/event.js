import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { onCleanup, onMount } from "solid-js";

/**
 * @param {(params: { close: () => void }) => void} callback
 */
export function onCloseRequested(callback) {
  let unlisten;

  onMount(() => {
    const window = getCurrentWebviewWindow();

    unlisten = window.listen("tauri://close-requested", () =>
      callback({
        close() {
          window.destroy();
        },
      }),
    );
  });

  onCleanup(async () => (await unlisten)?.());
}
