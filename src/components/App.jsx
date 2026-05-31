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
import styles from "@/styles/App";
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
  FunnelPlus,
  PanelBottomClose,
  PanelBottomOpen,
  PanelLeftClose,
  PanelLeftOpen,
  PenLine,
  Trash,
} from "lucide-solid";

export default function App() {
  const {
    views,
    getView,
    setViewDefinition: setViewDefinitionImpl,
    createView: createViewImpl,
    removeView: removeViewImpl,
  } = useViews();

  const [currentViewId, setCurrentViewId] = createSignal(null);
  const currentView = createMemo(() => getView(currentViewId()));

  const { entries, refetchEntries, setEntryContent } =
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

  return (
    <div id="app" class={styles.app}>
      <Workspace>
        <Panel
          orientation="horizontal"
          collapsed={leftPanelCollapsed()}
          controls={
            <PanelControls placement="top" sticky="right">
              <IconButton
                icon={leftPanelCollapsed() ? PanelLeftOpen : PanelLeftClose}
                onClick={toggleLeftPanelCollapsed}
              />
            </PanelControls>
          }
        >
          <Label style="panel">Views</Label>
          <Tabs currentId={currentViewId()} onCurrentChange={setCurrentViewId}>
            <Key each={views()} by="id">
              {(view) => (
                <Tab
                  id={view().id}
                  title={view().name}
                  controls={
                    <Show when={view().id !== defaultView.id}>
                      <IconButton size="small" icon={PenLine} />
                      <IconButton
                        size="small"
                        style="danger"
                        icon={Trash}
                        onClick={() => removeView(view().id)}
                      />
                    </Show>
                  }
                >
                  {/* TODO: Show total entry count */}
                  <Show when={view().id !== defaultView.id}>
                    <Badge size="small" fixedWidth>
                      {view().entryCount}
                    </Badge>
                  </Show>
                </Tab>
              )}
            </Key>
          </Tabs>
          <PanelBottomContainer>
            <Button size="wide" icon={FunnelPlus} onClick={createView}>
              New
            </Button>
          </PanelBottomContainer>
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
              </Panel>
            </Show>
          }
        >
          <Show when={leftPanelCollapsed()}>
            <EntriesHeader>
              Header
              <Badge size="large">42</Badge>
            </EntriesHeader>
          </Show>
          <Entries>
            <Key each={entries()} by="id">
              {(entry) => (
                <Entry
                  timestamp={entry().createdAt}
                  content={entry().content}
                  onContentChange={(content) =>
                    setEntryContent(entry().id, content)
                  }
                />
              )}
            </Key>
          </Entries>
        </EntriesContainer>
      </Workspace>
    </div>
  );
}
