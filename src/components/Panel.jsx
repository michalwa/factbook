import { Show } from "solid-js";
import styles from "@/styles/Panel";
import Label from "@/components/Label";

export default function Panel(props) {
  const styleClass = () => props.style && styles[`style-${props.style}`];
  const orientationClass = () =>
    props.orientation && styles[`orientation-${props.orientation}`];

  return (
    <div
      class={`${styles.panel} ${styleClass()} ${orientationClass()}`}
      {...(props.collapsed && { "data-collapsed": "" })}
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
