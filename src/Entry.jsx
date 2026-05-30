import { Match, Switch } from "solid-js";
import CodeEditor from "./CodeEditor";
import styles from "./Entry.module.css";

export default function Entry(props) {
  return (
    <div class={styles.entry}>
      <time class={styles.timestamp} datetime={props.timestamp}>
        {props.timestamp}
      </time>
      <div class={styles.divider}></div>
      <Switch>
        <Match when={props.mode === "static"}>
          <p class={styles.content}>{props.content}</p>
        </Match>
        <Match when={props.mode === "editor"}>
          <CodeEditor class={styles.content} value={props.content} />
        </Match>
      </Switch>
    </div>
  );
}
