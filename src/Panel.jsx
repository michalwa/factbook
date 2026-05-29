import { Show } from "solid-js";
import styles from "./Panel.module.css";
import Label from "./Label";

export default function Panel(props) {
  return (
    <div
      class={`${styles.panel} ${props.orientation && styles[`orientation-${props.orientation}`]}`}
      {...(props.expanded && { "data-expanded": "" })}
    >
      <div class={styles.content}>
        <Show when={props.label}>
          <Label class={styles.label}>{props.label}</Label>
        </Show>
        {props.children}
      </div>
      {props.controls}
    </div>
  );
}
