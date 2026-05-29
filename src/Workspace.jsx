import styles from "./Workspace.module.css";

export default function Workspace(props) {
  return <div class={styles.container}>{props.children}</div>;
}
