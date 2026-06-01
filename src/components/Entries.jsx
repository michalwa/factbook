import styles from "@/styles/Entries";
import TransitionGroup from "@/components/TransitionGroup";

export default function Entries(props) {
  return (
    <div class={styles.entries}>
      <TransitionGroup>{props.children}</TransitionGroup>
    </div>
  );
}
