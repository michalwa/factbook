import styles from "@/styles/EntryList";
import Entry, { emitEntryFocusRequested } from "@/components/Entry";
import TransitionGroup from "@/components/TransitionGroup";
import { neighbors } from "@/utils";
import { Key } from "@solid-primitives/keyed";
import { destructure } from "@solid-primitives/destructure";
import { Plus } from "lucide-solid";
import IconButton from "@/components/IconButton";

export default function createEntryList() {
  let ref;

  const focusEntry = (data) => emitEntryFocusRequested(ref, data);

  const EntryList = (props) => (
    <div ref={ref} class={styles.list}>
      <TransitionGroup>
        <Key
          each={neighbors(props.entries)}
          by={([prev, entry, next]) => entry.id}
        >
          {(neighbors) => {
            const [prev, entry, next] = destructure(neighbors);
            return (
              <Entry
                parentRef={ref}
                id={entry().id}
                timestamp={entry().createdAt}
                content={entry().content}
                spans={entry().spans}
                parseSpans={props.parseEntryContent}
                onContentChange={(content) =>
                  props.onContentChange(entry().id, content)
                }
                onRemove={() => {
                  props.onRemove(entry().id);
                  prev() && focusEntry({ id: prev().id, direction: "up" });
                }}
                onNavigateUp={(data) =>
                  prev() && focusEntry({ id: prev().id, ...data })
                }
                onNavigateDown={(data) =>
                  next() && focusEntry({ id: next().id, ...data })
                }
                onFocus={() => props.onFocus(entry().id)}
              />
            );
          }}
        </Key>
        <div class={styles.entryContentMargin}>
          <IconButton icon={Plus} onClick={props.onCreate} />
        </div>
      </TransitionGroup>
    </div>
  );

  return { EntryList, focusEntry };
}
