import styles from "@/styles/Workspace";

export default function Workspace(props) {
  return <div class={styles.container}>{props.children}</div>;
}
