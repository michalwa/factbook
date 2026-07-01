import Badge from "@/components/Badge";
import Button from "@/components/Button";
import createEntryList from "@/components/EntryList";
import EntryListContainer from "@/components/EntryListContainer";
import EntryListHeader from "@/components/EntryListHeader";
import IconButton from "@/components/IconButton";
import Label from "@/components/Label";
import Panel from "@/components/Panel";
import PanelBottomContainer from "@/components/PanelBottomContainer";
import PanelControls from "@/components/PanelControls";
import PanelControlsSpacer from "@/components/PanelControlsSpacer";
import styles from "@/styles/Journal";
import Tab from "@/components/Tab";
import Tabs from "@/components/Tabs";
import { useEntries } from "@/api/entry";
import { useViews, defaultView } from "@/api/view";
import Workspace from "@/components/Workspace";
import createViewEditor from "@/components/ViewEditor";
import Status from "@/components/Status";
import { createMemo, createSignal, Show } from "solid-js";
import { Key } from "@solid-primitives/keyed";
import { useAppState } from "@/api/appState";
import { createTagsContext } from "@/api/tag";
import { useJournalSettings } from "@/api/store";
import {
  FilePlusCorner,
  FolderOpen,
  PanelLeftOpen,
  PanelLeftClose,
  PenLine,
  Trash,
  Check,
  X,
  FunnelPlus,
  PanelBottomOpen,
  PanelBottomClose,
  CircleQuestionMark,
} from "lucide-solid";
import { createHotkey } from "@tanstack/solid-hotkeys";

export default function Journal() {
  const { journalPath, createJournal, openJournal, openDefaultJournal } =
    useAppState();

  const { createJournalSetting } = useJournalSettings({ journalPath });
  const [leftPanelCollapsed, setLeftPanelCollapsed] = createJournalSetting(
    "left_panel_collapsed",
  );
  const [bottomPanelCollapsed, setBottomPanelCollapsed] = createJournalSetting(
    "bottom_panel_collapsed",
  );
  const [currentViewId, setCurrentViewId] =
    createJournalSetting("current_view_id");

  const { Provider: TagsContextProvider, refetchTags } = createTagsContext();

  const {
    views,
    getEditableView,
    getPreviousView,
    getNextView,
    setViewName,
    setViewDefinition: setViewDefinitionImpl,
    createView: createViewImpl,
    removeView: removeViewImpl,
  } = useViews();

  const {
    entries,
    refetchEntries,
    setEntryContent: setEntryContentImpl,
    parseEntryContent,
    createEntry: createEntryImpl,
    removeEntry,
  } = useEntries(currentViewId);

  const currentEditableView = createMemo(() =>
    getEditableView(currentViewId()),
  );

  const setViewDefinition = async (...args) => {
    await setViewDefinitionImpl(...args);
    refetchEntries();
    refetchTags();
  };
  const createView = async () => setCurrentViewId(await createViewImpl());
  const removeView = (...args) => {
    setCurrentViewId(defaultView.id);
    return removeViewImpl(...args);
  };

  const [lastFocusedEntryId, setLastFocusedEntryId] = createSignal();
  const { EntryList, focusEntry } = createEntryList();

  const setEntryContent = async (...args) => {
    const result = await setEntryContentImpl(...args);
    refetchTags();
    return result;
  };
  const createEntry = async () => {
    const id = await createEntryImpl();
    console.log(id);
    focusEntry({ id });
  };

  const {
    ViewEditor,
    focus: focusViewEditor,
    hasFocus: viewEditorHasFocus,
  } = createViewEditor();

  createHotkey("Mod+Enter", () => createEntry());
  createHotkey("Mod+B", () => setLeftPanelCollapsed(!leftPanelCollapsed()));
  createHotkey("Mod+E", () => {
    if (viewEditorHasFocus() || !currentEditableView()) {
      setBottomPanelCollapsed(true);
      const entryId = lastFocusedEntryId() ?? entries()?.[0]?.id;
      if (entryId !== undefined) focusEntry({ id: entryId });
    } else {
      setBottomPanelCollapsed(false);
      focusViewEditor();
    }
  });
  createHotkey("Mod+PageUp", () => {
    const prev = getPreviousView(currentViewId());
    prev && setCurrentViewId(prev.id);
  });
  createHotkey("Mod+PageDown", () => {
    const next = getNextView(currentViewId());
    next && setCurrentViewId(next.id);
  });

  return (
    <TagsContextProvider>
      <Workspace>
        <Panel
          orientation="horizontal"
          collapsed={leftPanelCollapsed()}
          controls={
            <>
              <PanelControls placement="top" sticky="right">
                <IconButton
                  icon={leftPanelCollapsed() ? PanelLeftOpen : PanelLeftClose}
                  onClick={() => setLeftPanelCollapsed(!leftPanelCollapsed())}
                />
              </PanelControls>
              <PanelControls placement="bottom" sticky="right">
                <IconButton
                  icon={CircleQuestionMark}
                  onClick={openDefaultJournal}
                  title="Help"
                />
              </PanelControls>
              <Show when={!leftPanelCollapsed()}>
                <PanelControls placement="bottom left">
                  <IconButton
                    icon={FilePlusCorner}
                    onClick={createJournal}
                    title="New journal"
                  />
                  <IconButton
                    icon={FolderOpen}
                    onClick={openJournal}
                    title="Open journal"
                  />
                </PanelControls>
              </Show>
            </>
          }
        >
          <Label style="panel">Views</Label>
          <Tabs currentId={currentViewId()} onCurrentChange={setCurrentViewId}>
            <Key each={views()} by="id">
              {(view) => (
                <Tab
                  id={view().id}
                  title={view().name || "(untitled)"}
                  onTitleChange={(title) => setViewName(view().id, title)}
                  controls={({
                    editTitle,
                    editingTitle,
                    saveTitle,
                    resetTitle,
                  }) => (
                    <>
                      <Show
                        when={view().id !== defaultView.id && !editingTitle()}
                      >
                        <IconButton
                          size="small"
                          icon={PenLine}
                          onClick={editTitle}
                          title="Rename"
                        />
                        <IconButton
                          size="small"
                          style="danger"
                          icon={Trash}
                          onClick={() => removeView(view().id)}
                          title="Delete"
                        />
                      </Show>
                      <Show when={editingTitle()}>
                        <IconButton
                          size="small"
                          icon={Check}
                          onClick={saveTitle}
                          title="Save"
                        />
                        <IconButton
                          size="small"
                          style="danger"
                          icon={X}
                          onClick={resetTitle}
                          title="Cancel"
                        />
                      </Show>
                    </>
                  )}
                >
                  {({ editingTitle }) => (
                    // TODO: Show total entry count
                    <Show
                      when={view().id !== defaultView.id && !editingTitle()}
                    >
                      <Badge size="small" fixedWidth>
                        {view().entryCount}
                      </Badge>
                    </Show>
                  )}
                </Tab>
              )}
            </Key>
          </Tabs>
          <PanelBottomContainer>
            <Button
              size="wide"
              icon={FunnelPlus}
              onClick={createView}
              title="New view"
            >
              New
            </Button>
          </PanelBottomContainer>
          <PanelControlsSpacer />
        </Panel>
        <EntryListContainer
          after={
            <Show when={currentEditableView()}>
              <Panel
                orientation="vertical"
                collapsed={bottomPanelCollapsed()}
                controls={
                  <PanelControls
                    placement="right"
                    sticky="top"
                    class={bottomPanelCollapsed() ? styles.statusMargin : ""}
                  >
                    <IconButton
                      icon={
                        bottomPanelCollapsed()
                          ? PanelBottomOpen
                          : PanelBottomClose
                      }
                      onClick={() =>
                        setBottomPanelCollapsed(!bottomPanelCollapsed())
                      }
                    />
                  </PanelControls>
                }
              >
                <Label style="panel">Edit view</Label>
                <ViewEditor
                  definition={currentEditableView().definition}
                  onDefinitionChange={(definition) =>
                    setViewDefinition(currentViewId(), definition)
                  }
                />
                <PanelControlsSpacer when={leftPanelCollapsed()} />
              </Panel>
            </Show>
          }
        >
          {/* TODO: Show total entry count */}
          <Show when={leftPanelCollapsed() && currentEditableView()}>
            <EntryListHeader>
              {currentEditableView().name || "(untitled)"}
              <Badge size="large">{currentEditableView().entryCount}</Badge>
            </EntryListHeader>
          </Show>
          <EntryList
            entries={entries()}
            parseEntryContent={parseEntryContent}
            onCreate={createEntry}
            onRemove={removeEntry}
            onFocus={setLastFocusedEntryId}
            onContentChange={setEntryContent}
          />
        </EntryListContainer>
        <Status />
      </Workspace>
    </TagsContextProvider>
  );
}
