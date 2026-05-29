import styles from "./PanelBottomContainer.module.css";

export default function PanelBottomContainer(props) {
  return <div class={styles.container}>{props.children}</div>;
}
