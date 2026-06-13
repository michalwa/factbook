import Badge from "@/components/Badge";
import Button from "@/components/Button";
import Entries from "@/components/Entries";
import EntriesContainer from "@/components/EntriesContainer";
import EntriesHeader from "@/components/EntriesHeader";
import Entry from "@/components/Entry";
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
import ViewEditor from "@/components/ViewEditor";
import { createMemo, createSignal, Show } from "solid-js";
import { createToggle } from "@/utils";
import { Key } from "@solid-primitives/keyed";
import {
  Check,
  FunnelPlus,
  LogOut,
  PanelBottomClose,
  PanelBottomOpen,
  PanelLeftClose,
  PanelLeftOpen,
  PenLine,
  Plus,
  Trash,
  X,
} from "lucide-solid";
import { useAppState } from "@/api/appState";

export default function Journal() {
  const { closeJournal } = useAppState();
  const {
    views,
    getView,
    setViewName,
    setViewDefinition: setViewDefinitionImpl,
    createView: createViewImpl,
    removeView: removeViewImpl,
  } = useViews();

  const [currentViewId, setCurrentViewId] = createSignal(null);
  const currentView = createMemo(() => getView(currentViewId()));

  const { entries, refetchEntries, setEntryContent, createEntry, removeEntry } =
    useEntries(currentViewId);

  const setViewDefinition = async (...args) => {
    await setViewDefinitionImpl(...args);
    refetchEntries();
  };
  const createView = async () => setCurrentViewId(await createViewImpl());
  const removeView = (...args) => {
    setCurrentViewId(defaultView.id);
    return removeViewImpl(...args);
  };

  const [leftPanelCollapsed, toggleLeftPanelCollapsed] = createToggle();
  const [bottomPanelCollapsed, toggleBottomPanelCollapsed] = createToggle();

  const onEntryContentChange =
    (entry) =>
    async (content, { setTokens }) => {
      const updatedEntry = await setEntryContent(entry().id, content);
      setTokens(updatedEntry.tokens);
    };

  return (
    <Workspace>
      <Panel
        orientation="horizontal"
        collapsed={leftPanelCollapsed()}
        controls={
          <>
            <PanelControls placement="top" sticky="right">
              <IconButton
                icon={leftPanelCollapsed() ? PanelLeftOpen : PanelLeftClose}
                onClick={toggleLeftPanelCollapsed}
              />
            </PanelControls>
            <PanelControls placement="bottom" sticky="right">
              <IconButton
                style="danger"
                flip="horizontal"
                icon={LogOut}
                onClick={closeJournal}
              />
            </PanelControls>
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
                      />
                      <IconButton
                        size="small"
                        style="danger"
                        icon={Trash}
                        onClick={() => removeView(view().id)}
                      />
                    </Show>
                    <Show when={editingTitle()}>
                      <IconButton
                        size="small"
                        icon={Check}
                        onClick={saveTitle}
                      />
                      <IconButton
                        size="small"
                        style="danger"
                        icon={X}
                        onClick={resetTitle}
                      />
                    </Show>
                  </>
                )}
              >
                {({ editingTitle }) => (
                  // TODO: Show total entry count
                  <Show when={view().id !== defaultView.id && !editingTitle()}>
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
          <Button size="wide" icon={FunnelPlus} onClick={createView}>
            New
          </Button>
        </PanelBottomContainer>
        <PanelControlsSpacer />
      </Panel>
      <EntriesContainer
        after={
          <Show when={currentViewId() !== defaultView.id}>
            <Panel
              orientation="vertical"
              collapsed={bottomPanelCollapsed()}
              controls={
                <PanelControls placement="right" sticky="top">
                  <IconButton
                    icon={
                      bottomPanelCollapsed()
                        ? PanelBottomOpen
                        : PanelBottomClose
                    }
                    onClick={toggleBottomPanelCollapsed}
                  />
                </PanelControls>
              }
            >
              <Label style="panel">Edit view</Label>
              <ViewEditor
                definition={currentView().definition}
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
        <Show when={leftPanelCollapsed() && currentViewId() !== defaultView.id}>
          <EntriesHeader>
            {currentView().name || "(untitled)"}
            <Badge size="large">{currentView().entryCount}</Badge>
          </EntriesHeader>
        </Show>
        <Entries>
          <Key each={entries()} by="id">
            {(entry) => (
              <Entry
                timestamp={entry().createdAt}
                content={entry().content}
                onContentChange={onEntryContentChange(entry)}
                onRemove={() => removeEntry(entry().id)}
                tokens={entry().tokens}
              />
            )}
          </Key>
          <IconButton
            icon={Plus}
            class={styles.entryContentMargin}
            onClick={createEntry}
          />
        </Entries>
      </EntriesContainer>
    </Workspace>
  );
}
