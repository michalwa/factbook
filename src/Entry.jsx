import styles from "./Entry.module.css";

export default function Entry(props) {
  return (
    <div class={styles.entry}>
      <time class={styles.timestamp} datetime={props.timestamp}>
        {props.timestamp}
      </time>
      <div class={styles.divider}></div>
      <p class={styles.content}>{props.content}</p>
    </div>
  );
}
