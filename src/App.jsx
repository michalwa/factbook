import Entries from "./Entries";
import Panel from "./Panel";
import Workspace from "./Workspace";
import styles from "./App.module.css";
import Label from "./Label";
import {
  ArrowRight,
  Banana,
  CircleQuestionMark,
  PanelBottomClose,
  PanelBottomOpen,
  PanelLeftClose,
  PanelLeftOpen,
  PenLine,
  Plus,
  Settings,
  Trash,
  TriangleAlert,
} from "lucide-solid";
import Button from "./Button";
import IconButton from "./IconButton";
import Input from "./Input";
import Tabs from "./Tabs";
import Tab from "./Tab";
import Badge from "./Badge";
import TabControls from "./TabControls";
import EntriesContainer from "./EntriesContainer";
import PanelControls from "./PanelControls";
import { createToggle } from "./utils";
import { Show } from "solid-js";
import EntriesHeader from "./EntriesHeader";
import PanelBottomContainer from "./PanelBottomContainer";
import PanelControlsSpacer from "./PanelControlsSpacer";

export default function App() {
  const [leftPanelExpanded, toggleLeftPanelExpanded] = createToggle();
  const [bottomPanelExpanded, toggleBottomPanelExpanded] = createToggle();

  return (
    <div id="app" class={styles.app}>
      <Workspace>
        <Panel
          orientation="horizontal"
          expanded={leftPanelExpanded()}
          controls={
            <>
              <PanelControls placement="top" sticky="right">
                <IconButton
                  icon={leftPanelExpanded() ? PanelLeftClose : PanelLeftOpen}
                  onClick={toggleLeftPanelExpanded}
                />
              </PanelControls>
              <PanelControls placement="bottom" sticky="right">
                <IconButton icon={CircleQuestionMark} />
                <IconButton icon={Settings} />
              </PanelControls>
            </>
          }
        >
          <Label style="panel">Side panel</Label>
          Hello, world!
          <Label style="form">Buttons</Label>
          {/* TODO: For testing only, remove inline-styled elements afterwards */}
          <div style="display: flex; flex-flow: row nowrap; gap: 1rem">
            <div style="display: flex; flex-flow: column nowrap; gap: 0.5rem; width: fit-content">
              <Button style="primary" icon={ArrowRight} iconPlacement="right">
                Primary
              </Button>
              <Button style="danger" icon={TriangleAlert} iconPlacement="left">
                Danger
              </Button>
              <Button icon={Plus}>Default</Button>
            </div>
            <div style="display: flex; flex-flow: column nowrap; gap: 0.5rem; width: fit-content">
              <IconButton icon={Banana} />
              <IconButton style="danger" icon={Banana} />
            </div>
          </div>
          <Label style="form">Buttons</Label>
          <Input value="Lorem ipsum dolor sit amet" />
          <Tabs>
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
          </Tabs>
          <PanelBottomContainer>
            <Button size="wide" icon={Plus}>
              Create
            </Button>
          </PanelBottomContainer>
          <PanelControlsSpacer />
        </Panel>
        <EntriesContainer
          after={
            <Panel
              orientation="vertical"
              expanded={bottomPanelExpanded()}
              controls={
                <PanelControls placement="right" sticky="top">
                  <IconButton
                    icon={
                      bottomPanelExpanded() ? PanelBottomClose : PanelBottomOpen
                    }
                    onClick={toggleBottomPanelExpanded}
                  />
                </PanelControls>
              }
            >
              <Label style="panel">Bottom panel</Label>
              Hello, world!
              <PanelControlsSpacer when={!leftPanelExpanded()} />
            </Panel>
          }
        >
          <Show when={!leftPanelExpanded()}>
            <EntriesHeader>
              Header
              <Badge size="large">42</Badge>
            </EntriesHeader>
          </Show>
          <Entries />
        </EntriesContainer>
      </Workspace>
    </div>
  );
}
