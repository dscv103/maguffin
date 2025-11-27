# ADR-001: UI Framework Selection

## Status

**Accepted**

## Context

We need to select a UI framework for a cross-platform (macOS, Windows, Linux) desktop Git client built in Rust. The application requires:

- Rich, interactive UI with forms, lists, tree views, and diff visualization
- Cross-platform consistency and native feel
- Reasonable developer productivity and maintainability
- Integration with Rust backend for Git operations and GitHub API calls

### Options Considered

1. **Tauri + Web UI (React/Svelte)**
   - Rust backend with web-based frontend
   - Uses platform WebView for rendering
   - Established patterns from Electron ecosystem

2. **egui (Pure Rust)**
   - Immediate-mode GUI in pure Rust
   - GPU-accelerated rendering
   - Single codebase, no JavaScript

3. **Dioxus**
   - React-like Rust UI framework
   - Multiple renderers (desktop, web, mobile)
   - Still maturing

4. **Slint**
   - Declarative Rust-first UI
   - Native rendering
   - Commercial licensing for some use cases

## Decision

We will use **Tauri with a web-based frontend (React or Svelte)**.

## Rationale

### Advantages of Tauri + Web UI

| Factor | Assessment |
|--------|------------|
| **Developer Productivity** | Large talent pool familiar with React/Vue/Svelte. Extensive component libraries available (Radix, Tailwind, etc.). |
| **UI Polish** | Web technologies have mature tooling for creating polished, accessible interfaces. Design systems like Radix UI provide high-quality components. |
| **Cross-Platform Consistency** | WebView provides highly consistent rendering across platforms with minimal platform-specific code. |
| **Component Ecosystem** | Diff viewers (Monaco, CodeMirror), tree visualizations (D3, React Flow), and form components are readily available. |
| **Bundle Size** | Tauri produces smaller binaries than Electron (~5-10MB vs 50-150MB) by using platform WebView. |
| **Rust Integration** | Tauri provides ergonomic IPC between Rust backend and JS frontend with type safety via tauri-specta. |
| **Security** | Tauri's security model is stronger than Electron's by default (no Node.js in renderer). |

### Disadvantages and Mitigations

| Concern | Mitigation |
|---------|------------|
| **WebView Inconsistencies** | Windows uses Edge WebView2, macOS uses WebKit. Test thoroughly on all platforms. Avoid bleeding-edge CSS/JS features. |
| **Performance for Large Diffs** | Use virtualization for large file lists. Consider Monaco Editor which handles large files well. |
| **Memory Usage** | WebView has baseline memory cost (~50-100MB). Acceptable for a developer tool. |
| **Not "Pure Rust"** | The backend is pure Rust. The UI layer can be swapped in future if needed. |

### Why Not egui?

egui is excellent for developer tools and prototypes, but:

1. **Component Maturity**: Lacks polished, accessible form components out of the box
2. **Styling Complexity**: Achieving a modern, branded look requires significant effort
3. **Diff Viewer**: No equivalent to Monaco/CodeMirror for syntax-highlighted diffs
4. **Developer Pool**: Smaller contributor pool compared to web developers

egui remains a valid choice for a future lightweight companion tool or for users who prefer a more minimal interface.

### Why Not Dioxus?

Dioxus is promising but:

1. **Ecosystem Maturity**: Component ecosystem is still developing
2. **Desktop Renderer**: Desktop rendering is less battle-tested than Tauri
3. **Risk**: Newer project with less production usage data

## Consequences

### Positive

- Faster initial development with familiar web technologies
- Access to extensive UI component libraries
- Easier to find contributors with web development experience
- Smaller bundle size compared to Electron

### Negative

- Frontend requires JavaScript/TypeScript knowledge
- Two language runtimes (Rust + JS/TS) increase complexity
- WebView debugging requires browser devtools
- Platform WebView updates are outside our control

### Neutral

- Need to define clear IPC contract between Rust and JS
- Frontend build tooling (Vite, Webpack) required

## Implementation Notes

### Recommended Stack

```
Frontend:
├── Framework: React 18+ or Svelte 5
├── Build: Vite
├── Styling: Tailwind CSS
├── Components: Radix UI (if React) or Skeleton (if Svelte)
├── State: Zustand (React) or Svelte stores
├── Diff Viewer: Monaco Editor
└── Tree Viz: React Flow or custom SVG

Backend (Tauri):
├── IPC: Tauri commands with tauri-specta for type generation
├── Async: Tokio runtime
└── State: Arc<Mutex<T>> or Tauri's state management
```

### IPC Pattern

```rust
// Rust side
#[tauri::command]
async fn list_pull_requests(
    owner: String,
    repo: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<PullRequest>, Error> {
    state.pr_service.list(&owner, &repo).await
}
```

```typescript
// TypeScript side (auto-generated types via tauri-specta)
import { commands } from './bindings';

const prs = await commands.listPullRequests('owner', 'repo');
```

## References

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [tauri-specta](https://github.com/oscartbeaumont/tauri-specta)
- [egui](https://github.com/emilk/egui)
- [Tower App](https://www.git-tower.com/) - Reference for UI/UX

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2025-11-27 | Architecture Team | Initial decision |
