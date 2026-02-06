// `TransitionGroup` implementation copied and modified from
// https://github.com/solidjs-community/solid-transition-group/blob/17bb3f4d83deae62b3fbf5b76bf8e970865b5222/src/index.ts

/*
 * MIT License
 *
 * Copyright (c) 2020-2021 Ryan Carniato
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

import { createListTransition } from "@solid-primitives/transition-group";
import { resolveElements } from "@solid-primitives/refs";

// https://github.com/solidjs-community/solid-transition-group/issues/12
// for the css transition be triggered properly on firefox
// we need to wait for two frames before changeing classes
function nextFrame(fn) {
  requestAnimationFrame(() => requestAnimationFrame(fn));
}

function enterTransition(el) {
  el.classList.add("transition-group-enter-from", "transition-group-enter");

  nextFrame(() => {
    el.classList.remove("transition-group-enter-from");
    el.classList.add("transition-group-enter-to");

    el.addEventListener("transitionend", endTransition);
    el.addEventListener("animationend", endTransition);
  });

  function endTransition(e) {
    if (!e || e.target === el) {
      el.removeEventListener("transitionend", endTransition);
      el.removeEventListener("animationend", endTransition);

      el.classList.remove(
        "transition-group-enter-to",
        "transition-group-enter",
      );
    }
  }
}

function exitTransition(el, done) {
  // Don't animate element if it's not in the DOM
  // This can happen when elements are changed under Suspense
  if (!el.parentNode) return done?.();

  el.classList.add("transition-group-exit-from", "transition-group-exit");

  nextFrame(() => {
    el.classList.remove("transition-group-exit-from");
    el.classList.add("transition-group-exit-to");

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

      el.classList.remove("transition-group-exit-to", "transition-group-exit");
    }
  }
}

export default function TransitionGroup(props) {
  return createListTransition(resolveElements(() => props.children).toArray, {
    exitMethod: "remove",
    onChange({ added, removed, finishRemoved, list }) {
      for (const el of added) enterTransition(el);

      const toMove = [];

      for (const el of list) {
        if (
          el.isConnected &&
          (el instanceof HTMLElement || el instanceof SVGElement)
        ) {
          toMove.push({ el, rect: el.getBoundingClientRect() });
        }
      }

      // wait for the new list to be rendered
      queueMicrotask(() => {
        document.body.offsetHeight; // force reflow

        const moved = [];

        for (const { el, rect } of toMove) {
          if (el.isConnected) {
            const newRect = el.getBoundingClientRect(),
              dX = rect.left - newRect.left,
              dY = rect.top - newRect.top;

            if (dX || dY) {
              // set els to their old position before transition
              el.style.transform = `translate(${dX}px, ${dY}px)`;
              el.style.transitionDuration = "0s";
              moved.push(el);
            }
          }
        }

        document.body.offsetHeight; // force reflow

        for (const el of moved) {
          el.classList.add("transition-group-move");

          // clear transition - els will move to their new position
          el.style.transform = el.style.transitionDuration = "";

          el.addEventListener("transitionend", endTransition);

          function endTransition(e) {
            if (e.target === el || /transform$/.test(e.propertyName)) {
              el.removeEventListener("transitionend", endTransition);
              el.classList.remove(...classes.move);
            }
          }
        }
      });

      for (const el of removed) exitTransition(el, () => finishRemoved([el]));
    },
  });
};
