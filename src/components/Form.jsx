import styles from "@/styles/Form";

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
