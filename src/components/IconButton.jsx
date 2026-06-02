import { mergeProps } from "solid-js";
import { Dynamic } from "solid-js/web";
import styles from "@/styles/IconButton";
import genericStyles from "@/styles/Button";

export default function IconButton(props) {
  const merged = mergeProps({ style: "default" }, props);

  return (
    <button
      class={`${genericStyles.button} ${styles.button} ${styles[`style-${merged.style}`]} ${styles[`size-${merged.size}`]} ${props.class}`}
      type={props.type ?? "button"}
      onClick={props.onClick}
    >
      <Dynamic component={merged.icon} class={styles.icon} />
    </button>
  );
}
