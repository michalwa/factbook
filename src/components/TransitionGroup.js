// Heavily inspired by https://github.com/solidjs-community/solid-transition-group/blob/17bb3f4d83deae62b3fbf5b76bf8e970865b5222/src/index.ts

import { createListTransition } from "@solid-primitives/transition-group";
import { resolveElements } from "@solid-primitives/refs";
import styles from "@/styles/TransitionGroup";

// https://github.com/solidjs-community/solid-transition-group/issues/12
// for the css transition be triggered properly on firefox
// we need to wait for two frames before changeing classes
function nextFrame(fn) {
  requestAnimationFrame(() => requestAnimationFrame(fn));
}

function enterTransition(el) {
  el.classList.add(styles.enterFrom, styles.enter);

  nextFrame(() => {
    el.classList.remove(styles.enterFrom);
    el.classList.add(styles.enterTo);

    el.addEventListener("transitionend", endTransition);
    el.addEventListener("animationend", endTransition);
  });

  function endTransition(e) {
    if (!e || e.target === el) {
      el.removeEventListener("transitionend", endTransition);
      el.removeEventListener("animationend", endTransition);

      el.classList.remove(styles.enterTo, styles.enter);
    }
  }
}

function exitTransition(el, done) {
  // Don't animate element if it's not in the DOM
  // This can happen when elements are changed under Suspense
  if (!el.parentNode) return done?.();

  el.classList.add(styles.exitFrom, styles.exit);

  nextFrame(() => {
    el.classList.remove(styles.exitFrom);
    el.classList.add(styles.exitTo);

    el.addEventListener("transitionend", endTransition);
    el.addEventListener("animationend", endTransition);
  });

  function endTransition(e) {
    if (!e || e.target === el) {
      // calling done() will remove element from the DOM,
      // but also trigger onChange callback in <TransitionGroup>.
      // Which is why the classes need to removed afterwards,
      // so that removing them won't change el styles when for the move transition
      done?.();

      el.removeEventListener("transitionend", endTransition);
      el.removeEventListener("animationend", endTransition);

      el.classList.remove(styles.exitTo, styles.exit);
    }
  }
}

export default function TransitionGroup(props) {
  return createListTransition(resolveElements(() => props.children).toArray, {
    exitMethod: "move-to-end",
    onChange({ added, removed, finishRemoved, list }) {
      for (const el of added) enterTransition(el);

      const transformed = [];

      for (const el of list) {
        if (
          el.isConnected &&
          (el instanceof HTMLElement || el instanceof SVGElement)
        ) {
          transformed.push({
            el,
            rect: el.getBoundingClientRect(),
            // `exitMethod: "move-to-end"` will force remove elements down.
            // We manually keep them in place and allow them to fade out.
            // We can't use `"keep-index"`, because we want to get the moved
            // elements to assume their new positions, so that we can calculate
            // the offset transforms and animate them.
            strategy: removed.includes(el) ? "keep" : "move",
          });
        }
      }

      // wait for the new list to be rendered
      queueMicrotask(() => {
        document.body.offsetHeight; // force reflow

        const moved = [];
        const kept = [];

        for (const { el, rect, strategy } of transformed) {
          if (el.isConnected) {
            const newRect = el.getBoundingClientRect(),
              dX = rect.left - newRect.left,
              dY = rect.top - newRect.top;

            if (dX || dY) {
              if (strategy === "move") {
                // set els to their old position before transition
                el.style.transform = `translate(${dX}px, ${dY}px)`;
                moved.push(el);
              } else if (strategy === "keep") {
                // We want the exiting elements to be able to control their X
                // transform independent of this offset which is intended to keep
                // them in place
                el.style.setProperty("--transition-translate-y", `${dY}px`);
                kept.push(el);
              }

              el.style.transitionDuration = "0s";
            }
          }
        }

        void document.body.offsetHeight; // force reflow

        for (const el of moved) {
          el.classList.add(styles.move);

          // clear transition - els will move to their new position
          el.style.transform = "";
          el.style.transitionDuration = "";

          el.addEventListener("transitionend", endTransition);

          function endTransition(e) {
            if (e.target === el || /transform$/.test(e.propertyName)) {
              el.removeEventListener("transitionend", endTransition);
              el.classList.remove(styles.move);
            }
          }
        }

        for (const el of kept) {
          el.style.transitionDuration = "";
        }
      });

      for (const el of removed) exitTransition(el, () => finishRemoved([el]));
    },
  });
}
