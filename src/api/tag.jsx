import { throttle } from "@solid-primitives/scheduled";
import { invoke } from "@tauri-apps/api/core";
import { createContext, createResource, useContext } from "solid-js";

const TagsContext = createContext();

export function createTagsContext() {
  const [tags, { refetch: refetchTags }] = createResource(() =>
    invoke("get_tags"),
  );

  const refetchTagsThrottled = throttle(refetchTags, 500);

  const context = {
    tags,
    refetchTags: refetchTagsThrottled,
  };

  const Provider = (props) => (
    <TagsContext.Provider value={context}>
      {props.children}
    </TagsContext.Provider>
  );

  return { ...context, Provider };
}

export function useTags() {
  return useContext(TagsContext);
}
