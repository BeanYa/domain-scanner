# Domain Scanner Design System

## 1. Product Direction

Domain Scanner is a professional desktop application for high-volume domain discovery, scanning, filtering, vector search, and LLM-assisted analysis. The interface should feel precise, operational, and calm under load. It is a tool for scanning large candidate sets, monitoring long-running jobs, and inspecting result quality, so the visual system favors dense readable data, predictable controls, and clear state communication.

The product aesthetic is dark, editorial, and utilitarian. It uses restrained surfaces, fine borders, compact typography, and monochrome structure with muted semantic accents. Visual weight should come from live metrics, tables, logs, charts, progress, and domain result cards rather than decorative illustration.

## 2. Core Principles

- **Operational clarity**: Every screen should answer what is running, what changed, what needs attention, and what action is available.
- **Dense but readable**: Use compact spacing for tables, logs, and metrics without sacrificing scannability.
- **Calm status language**: Reserve color for scan state, risk, success, warnings, and destructive actions.
- **No borrowed branding**: UI copy, navigation, and examples must refer to Domain Scanner concepts only.
- **No decorative shadows**: Use background contrast, borders, and spacing for hierarchy.
- **Stable layouts**: Progress, counters, task rows, and controls must not resize when values update.
- **Accessible focus**: Keyboard focus must remain visible on all interactive elements.

## 3. Color Palette

### Backgrounds

| Token | Hex | Role |
| --- | --- | --- |
| `cyber.bg` | `#000000` | App background |
| `cyber.bg-elevated` | `#030303` | Raised panels and page sections |
| `cyber.surface` | `#0c0c0c` | Inputs, dense containers, table headers |
| `cyber.card` | `#111111` | Cards and row hover base |
| `cyber.card-hover` | `#1a1a1a` | Active and hover surfaces |

### Borders And Text

| Token | Hex | Role |
| --- | --- | --- |
| `cyber.border` | `#27272a` | Default border |
| `cyber.border-light` | `#3f3f3f` | Hover border and secondary dividers |
| `cyber.text` | `#ffffff` | Primary text |
| `cyber.text-secondary` | `#c9ccd1` | Secondary text and values |
| `cyber.muted` | `#767d88` | Labels, helper copy, metadata |
| `cyber.muted-dim` | `#565d66` | Placeholders and disabled detail |

### Semantic Accents

| Token | Hex | Use |
| --- | --- | --- |
| `cyber.green` | `#e8ece8` | Running, available, success |
| `cyber.blue` | `#dce2ea` | Completed, informational |
| `cyber.orange` | `#d8c7a4` | Paused, pending, warning |
| `cyber.red` | `#e2aaa6` | Error, failed, destructive |
| `cyber.purple` | `#d0d4d4` | Vector/LLM secondary accent |

Use accent colors sparingly. Most UI should remain black, near-black, white, and cool gray.

## 4. Typography

### Font Stack

The project uses a deterministic system font stack rather than a proprietary custom font:

```css
font-family: "Aptos", "Segoe UI", system-ui, sans-serif;
```

Monospace content uses:

```css
font-family: "JetBrains Mono", "Cascadia Mono", Consolas, monospace;
```

This keeps screenshots and production builds stable across developer machines without adding a font asset or license requirement.

### Type Scale

| Role | Size | Weight | Line Height | Letter Spacing | Typical Use |
| --- | --- | --- | --- | --- | --- |
| Page heading | 32-40px | 400 | 1.0 | 0 | Dashboard and page titles |
| Section title | 16-20px | 500-600 | 1.25 | 0 | Panel headings |
| Body | 14px | 400 | 1.5 | 0 | Forms, descriptions, row text |
| Dense body | 13px | 400 | 1.35 | 0 | Tables, result metadata |
| Label | 11-12px | 500-600 | 1.3 | 0 | Uppercase labels, tags |
| Numeric metric | 24-32px | 400-500 | 1.0 | 0 | Counts, rates, totals |
| Log/Domain | 12-13px | 400 | 1.4 | 0 | Logs, domain names, IDs |

The current base stylesheet intentionally normalizes `letter-spacing` to `0`. Do not rely on Tailwind `tracking-*` utilities unless that global rule is removed in a dedicated style-system change.

## 5. Layout

### App Shell

- Left sidebar for persistent navigation: Dashboard, Tasks, New Task, Filter, Vectorize, Proxies, Settings.
- Main content uses constrained horizontal padding and full-height scroll behavior.
- Page headers should include a clear title, short supporting text, and primary action when relevant.
- Keep scan controls close to the data they affect.

### Spacing

- Base unit: 4px.
- Common gaps: 8px, 12px, 16px, 20px, 24px.
- Page section gaps: 24px.
- Dense table/log padding: 8-12px.
- Form group spacing: 16px.

### Containers

- Use `editorial-panel` for major panels.
- Use `metric-tile` for compact statistics.
- Use `task-card` for navigable task summaries.
- Avoid nesting full cards inside other full cards.
- Prefer full-width panels for tables, logs, and charts.

## 6. Components

### Navigation

- The sidebar is the primary navigation surface.
- Active route should use stronger text contrast and a subtle surface change.
- Icons should support quick scanning, but labels remain visible on desktop.
- Do not use external product wordmarks or brand references.

### Buttons

- Primary action: white background, black text, solid border.
- Secondary action: transparent background, muted text, thin border.
- Ghost action: text-only with subtle hover surface.
- Danger action: red text and red-tinted border.
- Buttons use 4-8px radius, never pill-shaped.
- Icon-only buttons need stable square dimensions and accessible labels.

### Forms

- Inputs use dark surfaces with visible borders.
- Placeholder text uses `cyber.muted-dim`.
- Validation states use semantic border color and concise helper text.
- Long-running actions must expose disabled and loading states.

### Tables And Result Lists

- Tables are first-class UI, not secondary content.
- Header rows use `cyber.surface`, uppercase labels, and muted text.
- Body rows use compact padding and visible hover state.
- Domain names, task IDs, run IDs, and log fragments may use the monospace stack.
- Pagination must keep controls stable as totals change.

### Task Cards

Task cards should show:

- Task name or generated scan label.
- Current status.
- Candidate count, completed count, available count, and error count when available.
- Last update time.
- Progress bar when running or resumable.
- Direct actions only when they are safe and expected from the list view.

### Progress

- Progress bars use low-height tracks with white or semantic fills.
- Show numeric progress when available.
- Indeterminate states should animate subtly and avoid layout shifts.
- High-volume scans should prefer throttled updates over rapid visual churn.

### Logs

- Logs are operational evidence and should remain readable.
- Use monospace text, compact line-height, and muted timestamps.
- Keep severity color minimal.
- Tail windows should not cause the surrounding layout to jump.

### Charts And Metrics

- Charts should support comparison and trend inspection, not decoration.
- Use muted grid lines and high-contrast values.
- Avoid saturated chart palettes unless semantic differences require them.
- Metric tiles should be compact and sortable by importance.

### Empty States

Empty states must explain the immediate next action in product terms:

- No tasks: create a new scan.
- No results: wait for scan progress or adjust filters.
- No proxies: add a proxy or continue without one.
- No vector index: vectorize results before semantic filtering.

Do not use generic marketing copy or unrelated mission statements.

## 7. Responsive Behavior

| Width | Behavior |
| --- | --- |
| `<640px` | Single-column panels, compact controls, stacked metrics |
| `640-1024px` | Two-column metric grids and simplified table columns |
| `>1024px` | Full dashboard layout with sidebar and dense tables |

Touch targets should remain at least 40px tall where possible. Dense tables can use compact row height, but row actions must remain easy to target.

## 8. Accessibility And State

- Use visible focus rings on keyboard navigation.
- Do not communicate scan status with color alone.
- Keep contrast high on dark surfaces.
- Use `aria-live` only for important status changes, not every progress tick.
- Preserve user control during long scans with pause, resume, and cancel states where supported.
- Loading skeletons should reserve the final layout size.

## 9. Implementation Notes

- Tailwind tokens in `tailwind.config.js` are the source of truth for color and font families.
- Shared component classes live in `src/index.css`.
- Avoid custom one-off colors in React components unless a new token is intentionally added.
- Prefer existing classes such as `cyber-btn`, `cyber-input`, `badge`, `task-card`, `data-table`, `page-heading`, and `editorial-panel`.
- Keep border radius in the 4-8px range for routine UI.
- Use shadows only for unavoidable platform or boot states; do not introduce decorative panel shadows.

## 10. Agent Prompt Guide

When asking an agent to build or revise Domain Scanner UI, provide product-specific context:

- "Build a dense task dashboard for a desktop domain scanning app. Use dark panels, compact metrics, visible scan status, and stable progress rows."
- "Create a task detail results table with domain, TLD, availability, RDAP/DNS evidence, confidence, and paginated controls."
- "Design a proxy manager panel with health status, protocol, latency, last checked time, and safe destructive actions."
- "Create an empty state for no vector index that directs the user to run vectorization before semantic filtering."
- "Revise a settings form for GPU, embedding API, and LLM configuration using dark inputs, concise helper text, and visible validation states."

Do not ask for external brand references, hero-led marketing pages, external wordmarks, unrelated mission statements, or photography-led layouts for this product.
