import { mergeProps } from "solid-js";
import { Dynamic } from "solid-js/web";
import styles from "@/styles/IconButton";
import genericStyles from "@/styles/GenericButton";

export default function IconButton(props) {
  const merged = mergeProps({ style: "default" }, props);

  return (
    <button
      class={`
        ${genericStyles.button}
        ${styles.button}
        ${styles[`style-${merged.style}`]}
        ${merged.size && styles[`size-${merged.size}`]}
        ${merged.flip && styles[`flip-${merged.flip}`]}
        ${props.class}
      `}
      type={props.type ?? "button"}
      onClick={props.onClick}
      title={props.title}
    >
      <Dynamic component={merged.icon} class={styles.icon} />
    </button>
  );
}
