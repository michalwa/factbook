import createDialog from "@/components/Dialog";
import { createSignal, onMount } from "solid-js";
import Button from "@/components/Button";
import Panel from "@/components/Panel";
import Form from "@/components/Form";
import FormControls from "@/components/FormControls";
import FormField from "@/components/FormField";
import Label from "@/components/Label";
import FileInput from "@/components/FileInput";
import { ArrowRight } from "lucide-solid";
import { useAppState } from "@/api/appState";

export default function Start() {
  const { openJournal } = useAppState();
  const { Dialog, open } = createDialog();
  const [filePath, setFilePath] = createSignal();

  onMount(open);

  return (
    <Dialog>
      <Panel style="rounded">
        <Form>
          <FormField>
            <Label style="form">Journal file</Label>
            <FileInput
              filters={[{ name: "Journal file", extensions: ["json"] }]}
              onChange={setFilePath}
            />
          </FormField>
          <FormControls>
            <Button
              style="primary"
              icon={ArrowRight}
              iconPlacement="right"
              disabled={!filePath()}
              onClick={() => openJournal(filePath())}
            >
              Open
            </Button>
          </FormControls>
        </Form>
      </Panel>
    </Dialog>
  );
}
