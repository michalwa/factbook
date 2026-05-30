import styles from "./Entries.module.css";
import TransitionGroup from "./TransitionGroup";

export default function Entries(props) {
  return (
    <div class={styles.entries}>
      <TransitionGroup>{props.children}</TransitionGroup>
    </div>
  );
}
