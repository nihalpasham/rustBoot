# rustBoot Diagrams

This directory contains Mermaid diagrams for visual documentation.
GitHub renders Mermaid natively in markdown files. To view locally:

1. Open any `.md` file that includes these diagrams in GitHub
2. Use the [Mermaid CLI](https://mermaid.js.org/config/mermaid-cli.html):
   ```bash
   npx @mermaid-js/mermaid-cli mmdc -i boot-flow.mmd -o boot-flow.png
   ```

## Diagrams

| File | Type | Content |
|------|------|---------|
| `boot-flow.mmd` | Sequence diagram | Power-on to firmware jump flow |
| `state-machine.mmd` | State diagram | A/B boot state machine with transitions |
| `partition-layout.mmd` | Block diagram | Flash memory partition layout |
| `image-format.mmd` | Packet diagram | TLV image header byte layout |
| `crate-architecture.mmd` | Graph diagram | Crate dependency structure |

## Embedded In

- `docs/architecture/ARCHITECTURE.md` — All diagrams embedded
- `README.md` — Referenced in architecture section