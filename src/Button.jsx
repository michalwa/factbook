import { mergeProps, Show } from "solid-js";
import styles from "./Button.module.css";

export default function Button(props) {
  const merged = mergeProps({ style: "outline", iconPlacement: "left" }, props);

  const Icon = () => (
    <Dynamic
      component={merged.icon}
      class={`${styles.icon} ${styles[`iconPlacement-${merged.iconPlacement}`]}`}
    />
  );

  return (
    <button
      class={`${styles[`style-${merged.style}`]} ${merged.size && styles[`size-${merged.size}`]}`}
      type={props.type ?? "button"}
      onClick={props.onClick}
    >
      <Show when={merged.iconPlacement === "left"}>
        <Icon />
      </Show>
      {merged.children}
      <Show when={merged.iconPlacement === "right"}>
        <Icon />
      </Show>
    </button>
  );
}
