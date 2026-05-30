import styles from "@/styles/TabControls";

export default function TabControls(props) {
  return <span class={styles.container}>{props.children}</span>;
}
