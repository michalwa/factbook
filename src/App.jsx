import Entries from "./Entries";
import Panel from "./Panel";
import Workspace from "./Workspace";
import styles from "./App.module.css";
import Label from "./Label";
import {
  ArrowRight,
  Banana,
  Check,
  CircleQuestionMark,
  Lock,
  LockOpen,
  PanelBottomClose,
  PanelBottomOpen,
  PanelLeftClose,
  PanelLeftOpen,
  PenLine,
  Plus,
  Settings,
  Trash,
  TriangleAlert,
  X,
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
import { onMount, Show } from "solid-js";
import EntriesHeader from "./EntriesHeader";
import PanelBottomContainer from "./PanelBottomContainer";
import PanelControlsSpacer from "./PanelControlsSpacer";
import Entry from "./Entry";
import createDialog from "./Dialog";
import Form from "./Form";
import FormField from "./FormField";
import FormControls from "./FormControls";

export default function App() {
  const [leftPanelCollapsed, toggleLeftPanelCollapsed] = createToggle();
  const [bottomPanelCollapsed, toggleBottomPanelCollapsed] = createToggle();

  const { Dialog, open: openDialog } = createDialog();

  const [entryMode, toggleEntryMode] = createToggle("editor", "static");

  return (
    <div id="app" class={styles.app}>
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
              <Button
                style="primary"
                icon={ArrowRight}
                iconPlacement="right"
                onClick={openDialog}
              >
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
          <Label style="form">Input field</Label>
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
          <Button
            size="wide"
            icon={entryMode() === "static" ? Lock : LockOpen}
            onClick={toggleEntryMode}
          >
            Toggle editable entries
          </Button>
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
              <Label style="panel">Bottom panel</Label>
              Hello, world!
              <PanelControlsSpacer when={leftPanelCollapsed()} />
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
            <Entry
              mode={entryMode()}
              timestamp="2025-01-02 13:45"
              content="this is a longer entry which wraps into multiple lines. lorem ipsum dolor sit amet lorem ipsum dolor sit amet lorem ipsum dolor sit amet"
            />
            <Entry
              mode={entryMode()}
              timestamp="2025-02-03 14:56"
              content="this is a longer entry which wraps into multiple lines. lorem ipsum dolor sit amet lorem ipsum dolor sit amet lorem ipsum dolor sit amet"
            />
          </Entries>
        </EntriesContainer>
      </Workspace>
      <Dialog>
        {({ close: closeDialog }) => (
          <Panel
            style="rounded"
            controls={
              <PanelControls placement="top right">
                <IconButton style="danger" icon={X} onClick={closeDialog} />
              </PanelControls>
            }
          >
            <Label style="panel">Dialog</Label>
            <Form onSubmit={closeDialog}>
              <FormField>
                <Label style="form">First input field</Label>
                <Input />
              </FormField>
              <FormField>
                <Label style="form">Second input field</Label>
                <Input />
              </FormField>
              <FormControls>
                <Button
                  type="submit"
                  style="primary"
                  icon={Check}
                  iconPlacement="right"
                >
                  OK
                </Button>
              </FormControls>
            </Form>
          </Panel>
        )}
      </Dialog>
    </div>
  );
}
