import Entry from "./Entry";
import "./EntryList.css";

export default function EntryList() {
  return (
    <div class="entry-list">
      <Entry
        timestamp="2025-10-17 22:24"
        content="this is a random @thought and it’s a longer entry that wraps around into multiple lines. lorem ipsum dolor sit amet lorem ipsum dolor sit amet lorem ipsum dolor sit amet lorem ipsum dolor sit amet lorem ipsum dolor sit amet"
      />
      <Entry timestamp="2025-10-16 15:12" content="@todo walk the dog" />
      <Entry
        timestamp="2025-10-16 12:30"
        content="@todo buy milk @due(tomorrow)"
      />
    </div>
  );
}
