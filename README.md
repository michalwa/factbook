# factbook

Personal knowledge base app based on logic programming

## Philosophy

1. **Quick inbox** -- the most important feature is being able to dump _anything_ into your knowledge-base at any time, without any friction of organization. No need to think about which directory an entry belongs in, you do that later.
2. **Atomic entries** -- every thought is its own entity, organization comes second. Entries have no name by default, only a timestamp and text.
3. **Atomic tags** -- all organization is done via structured tags, which have no centralized definition, but whose semantics are defined by their usage, or by relevant views (queries). They are akin to Prolog terms. Creating ontologies is as simple as typing a tag inside of an entry, no need to do any meta-organization elsewhere. You are not forced to tag your entries, and if you do, they sit seamlessly scattered throughout your text.

   ```css
   walk the dog @todo @due(tomorrow) @priority(10)
   ```

4. **Views** (aka. queries, lenses) -- easily define subsets of your knowledge base by querying facts about entries, e.g. tags, timestamps. This is where organization happens, and there's no need to move any entries. You do it at your own pace, outside of the flow of taking notes. And you get all the [power of Prolog](https://www.metalevel.at/prolog) to your advantage.

   <!-- TODO: The example should ideally use existing predicates once they are implemented -->
   ```prolog
   % Example only, specific available predicates and semantics may differ
   show(E) :-
     tag(E, todo),
     tag(E, due(D)),
     created(E, D0),
     relative_datetime(D0, D, D1), % e.g. relative_datetime(2025-10-22, tomorrow, 2025-10-23).
     now(N),
     N #=< D1.
   ```
