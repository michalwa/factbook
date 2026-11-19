import styles from "@/styles/EntryList";
import Entry, { emitEntryFocusRequested } from "@/components/Entry";
import TransitionGroup from "@/components/TransitionGroup";
import { neighbors } from "@/utils";
import { Key } from "@solid-primitives/keyed";
import { destructure } from "@solid-primitives/destructure";
import { Plus } from "lucide-solid";
import IconButton from "@/components/IconButton";
import { createMemo } from "solid-js";

export default function createEntryList() {
  let ref;

  const focusEntry = (data) => emitEntryFocusRequested(ref, data);

  const EntryList = (props) => {
    const physicalNeighborIds = createMemo(
      () =>
        props.entries &&
        new Map(
          neighbors(props.entries.filter((e) => !e.synthetic)).map(
            ([prev, curr, next]) => [curr.id, [prev?.id, next?.id]],
          ),
        ),
    );

    return (
      <div ref={ref} class={styles.list}>
        <TransitionGroup>
          <Key each={props.entries} by={uniqueId}>
            {(entry) => {
              const [prevId, nextId] = physicalNeighborIds()?.get(entry().id) ?? [];

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
                    prevId !== undefined &&
                      focusEntry({ id: prevId, direction: "up" });
                  }}
                  onNavigateUp={(data) =>
                    prevId !== undefined && focusEntry({ id: prevId, ...data })
                  }
                  onNavigateDown={(data) =>
                    nextId !== undefined && focusEntry({ id: nextId, ...data })
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
  };

  return { EntryList, focusEntry };
}

function uniqueId(entry) {
  return (entry.synthetic ? "s" : "") + entry.id;
}
