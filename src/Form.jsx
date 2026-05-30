import styles from "./Form.module.css";

export default function Form(props) {
  return (
    <form
      class={styles.form}
      onSubmit={(e) => {
        e.preventDefault();
        props.onSubmit?.();
      }}
    >
      {props.children}
    </form>
  );
}
