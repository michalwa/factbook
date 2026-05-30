import styles from "@/styles/PanelBottomContainer";

export default function PanelBottomContainer(props) {
  return <div class={styles.container}>{props.children}</div>;
}
