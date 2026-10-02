# Issue tracker: Local Markdown

One effort per directory: `.scratch/<effort>/`.
Specifications live in `spec.md`; tickets live in
`issues/NN-<slug>.md`.

## Wayfinding operations

- Map: `.scratch/<effort>/map.md`, labelled `wayfinder:map`.
- Child tickets: numbered issue files, linked to their parent map.
- Metadata: Status, Labels, Type, Assignee, and Blocked by.
- Types: research, prototype, grilling, task.
- An open ticket is unclaimed when its Assignee is empty.
- Claim: set Assignee before beginning work.
- Blocking: list prerequisite ticket numbers in Blocked by.
  A ticket is unblocked when all prerequisites are closed.
- Frontier: open, unblocked, unclaimed children, ordered by number.
- Comments: append under a Comments heading.
- Resolve: append a resolution comment, close the ticket, and
  add a named link with a one-line gist to Decisions so far.
- Assets: link supporting files from the ticket.
- Refer to maps and tickets by their linked titles.
