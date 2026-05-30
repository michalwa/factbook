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
import { createSignal, Show } from "solid-js";
import EntriesHeader from "./EntriesHeader";
import PanelBottomContainer from "./PanelBottomContainer";
import PanelControlsSpacer from "./PanelControlsSpacer";
import Entry from "./Entry";
import createDialog from "./Dialog";
import Form from "./Form";
import FormField from "./FormField";
import FormControls from "./FormControls";
import { Key } from "@solid-primitives/keyed";
import { format as formatDate } from "date-fns";

export default function App() {
  const [leftPanelCollapsed, toggleLeftPanelCollapsed] = createToggle();
  const [bottomPanelCollapsed, toggleBottomPanelCollapsed] = createToggle();

  const { Dialog, open: openDialog } = createDialog();

  const [entries, setEntries] = createSignal([
    {
      id: 0,
      timestamp: "2025-01-02 13:45",
      content:
        "this is a longer entry which wraps into multiple lines. lorem ipsum dolor sit amet lorem ipsum dolor sit amet lorem ipsum dolor sit amet",
    },
    {
      id: 1,
      timestamp: "2025-02-03 14:56",
      content: "this is an example entry",
    },
  ]);
  const [nextEntryId, setNextEntryId] = createSignal(2);

  const insertEntry = () => {
    const content = [
      "this is a longer entry which wraps into multiple lines. lorem ipsum dolor sit amet lorem ipsum dolor sit amet lorem ipsum dolor sit amet",
      "this is an example entry",
      "lorem ipsum dolor sit amet lorem ipsum dolor sit amet lorem ipsum dolor sit amet",
    ];
    const entry = {
      id: nextEntryId(),
      timestamp: formatDate(new Date(), "yyyy-MM-dd hh:mm"),
      content: content[Math.floor(Math.random() * content.length)],
    };
    setEntries(
      entries().toSpliced(
        Math.floor(Math.random() * entries().length),
        0,
        entry,
      ),
    );
    setNextEntryId(entry.id + 1);
  };

  const deleteEntry = () => {
    if (entries().length) {
      setEntries(
        entries().toSpliced(Math.floor(Math.random() * entries().length), 1),
      );
    }
  };

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
          <Button size="wide" icon={Plus} onClick={insertEntry}>
            Insert entry
          </Button>
          <Button size="wide" icon={Trash} onClick={deleteEntry}>
            Delete entry
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
            <Key each={entries()} by="id">
              {(entry) => (
                <Entry
                  timestamp={entry().timestamp}
                  content={entry().content}
                />
              )}
            </Key>
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
