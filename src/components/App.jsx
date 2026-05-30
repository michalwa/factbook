import Entries from "@/components/Entries";
import Panel from "@/components/Panel";
import Workspace from "@/components/Workspace";
import styles from "@/styles/App";
import Label from "@/components/Label";
import {
  FunnelPlus,
  PanelBottomClose,
  PanelBottomOpen,
  PanelLeftClose,
  PanelLeftOpen,
} from "lucide-solid";
import Button from "@/components/Button";
import IconButton from "@/components/IconButton";
import Badge from "@/components/Badge";
import EntriesContainer from "@/components/EntriesContainer";
import PanelControls from "@/components/PanelControls";
import { createToggle } from "@//utils";
import { Show } from "solid-js";
import EntriesHeader from "@/components/EntriesHeader";
import PanelBottomContainer from "@/components/PanelBottomContainer";

export default function App() {
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
          {/* <Tabs>
            <Tab id={0} title="First tab">
              <TabControls>
                <IconButton size="small" icon={PenLine} />
                <IconButton size="small" style="danger" icon={Trash} />
              </TabControls>
              <Badge size="small" fixedWidth>
                1
              </Badge>
            </Tab>
            <Tab id={1} title="Second tab">
              <TabControls>
                <IconButton size="small" icon={PenLine} />
                <IconButton size="small" style="danger" icon={Trash} />
              </TabControls>
              <Badge size="small" fixedWidth>
                42
              </Badge>
            </Tab>
            <Tab id={2} title="Third tab">
              <TabControls>
                <IconButton size="small" icon={PenLine} />
                <IconButton size="small" style="danger" icon={Trash} />
              </TabControls>
              <Badge size="small" fixedWidth>
                123
              </Badge>
            </Tab>
          </Tabs> */}
          <PanelBottomContainer>
            <Button size="wide" icon={FunnelPlus}>
              New
            </Button>
          </PanelBottomContainer>
        </Panel>
        <EntriesContainer
          after={
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
            </Panel>
          }
        >
          <Show when={leftPanelCollapsed()}>
            <EntriesHeader>
              Header
              <Badge size="large">42</Badge>
            </EntriesHeader>
          </Show>
          <Entries>
            {/* <Key each={entries()} by="id">
              {(entry) => (
                <Entry
                  timestamp={entry().timestamp}
                  content={entry().content}
                />
              )}
            </Key>*/}
          </Entries>
        </EntriesContainer>
      </Workspace>
    </div>
  );
}
