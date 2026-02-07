import { createEffect } from "solid-js";
import "./Input.css";

export default function Input(props) {
  let input;

  createEffect(() => props.focus && input.focus());

  return (
    <form
      class="input-wrapper"
      action="javascript:void(0)"
      onSubmit={() => input.blur()}
    >
      <input
        ref={input}
        value={props.value ?? ""}
        onInput={() => props.onInput?.(input.value)}
      />
    </form>
  );
}
