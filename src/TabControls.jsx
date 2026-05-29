import styles from "./TabControls.module.css";

export default function TabControls(props) {
  return <span class={styles.container}>{props.children}</span>;
}
