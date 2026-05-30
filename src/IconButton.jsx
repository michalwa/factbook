import { mergeProps } from "solid-js";
import styles from "./IconButton.module.css";
import { Dynamic } from "solid-js/web";

export default function IconButton(props) {
  const merged = mergeProps({ style: "default" }, props);

  return (
    <button
      class={`${styles.button} ${styles[`style-${merged.style}`]} ${styles[`size-${merged.size}`]}`}
      type={props.type ?? "button"}
      onClick={props.onClick}
    >
      <Dynamic component={merged.icon} class={styles.icon} />
    </button>
  );
}
