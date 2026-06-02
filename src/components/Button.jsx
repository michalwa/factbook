import { mergeProps, Show } from "solid-js";
import styles from "@/styles/Button";
import genericStyles from "@/styles/GenericButton";

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
      class={`
        ${genericStyles.button}
        ${styles.button}
        ${styles[`style-${merged.style}`]}
        ${merged.size && styles[`size-${merged.size}`]}
      `}
      type={props.type ?? "button"}
      onClick={props.onClick}
      disabled={props.disabled}
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
