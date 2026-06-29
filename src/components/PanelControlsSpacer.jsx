import styles from "@/styles/PanelControlsSpacer";

export default function PanelControlsSpacer(props) {
  return (
    <div
      class={`${styles.spacer} ${!("when" in props) || props.when ? styles.expanded : styles.collapsed}`}
    />
  );
}
