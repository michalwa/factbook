import styles from "./Dialog.module.css";

export default function createDialog() {
  let el;

  const open = () => el.showModal();
  const close = () => el.close();

  const Dialog = (props) => (
    <dialog ref={el} class={styles.dialog}>
      {props.children({ close })}
    </dialog>
  );

  return { Dialog, open, close };
}
