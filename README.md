# Maguffin - Rust Desktop Git Client

A cross-platform desktop Git client built in Rust, featuring a Tower-style PR dashboard and Graphite-style stacked PR workflow for GitHub repositories.

## Features (Planned)

- **Tower-style PR Dashboard**: Browse and manage pull requests directly in the app
- **Graphite-style Stacked PRs**: Manage dependent branches with automatic restacking
- **GitHub Integration**: Full OAuth authentication and GraphQL API support
- **Cross-Platform**: Native binaries for macOS, Windows, and Linux

## Project Structure

```
maguffin/
├── REQUIREMENTS.md          # Functional and non-functional requirements
├── SOLUTIONPLAN.md          # Architecture and implementation plan
├── PROJECTSTATE.md          # SDLC phase tracking and handoff ledger
├── docs/
│   ├── Agent-Handoff-and-Ownership.md
│   └── adr/                 # Architecture Decision Records
│       ├── ADR-001-ui-framework-selection.md
│       └── ADR-002-git-integration-approach.md
└── maguffin-app/           # Tauri application
    ├── src/                # Frontend (TypeScript)
    ├── src-tauri/          # Backend (Rust)
    │   ├── src/
    │   │   ├── domain/     # Business logic and types
    │   │   │   ├── pr/     # Pull request types
    │   │   │   ├── stack/  # Stack management
    │   │   │   ├── auth/   # Authentication
    │   │   │   ├── repo/   # Repository types
    │   │   │   └── sync/   # Sync status
    │   │   ├── error/      # Error handling
    │   │   ├── config/     # Configuration
    │   │   ├── git/        # Git operations (git2)
    │   │   ├── github/     # GitHub GraphQL client
    │   │   ├── cache/      # SQLite cache
    │   │   ├── keyring/    # Secure credential storage
    │   │   └── commands/   # Tauri IPC commands
    │   └── Cargo.toml
    └── package.json
```

## Development

### Prerequisites

- Rust 1.70+
- Node.js 18+
- Platform-specific dependencies (see [Tauri Prerequisites](https://tauri.app/start/prerequisites/))

### Setup

```bash
# Clone the repository
git clone https://github.com/dscv103/maguffin.git
cd maguffin/maguffin-app

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Run Rust tests
npm run test:rust
```

### Building

```bash
# Build for release
npm run tauri build
```

## Architecture

See [SOLUTIONPLAN.md](SOLUTIONPLAN.md) for detailed architecture documentation.

Key technology choices:
- **UI Framework**: Tauri + Web UI (React/Svelte) - see [ADR-001](docs/adr/ADR-001-ui-framework-selection.md)
- **Git Integration**: git2 (libgit2) with CLI fallback - see [ADR-002](docs/adr/ADR-002-git-integration-approach.md)

## License

MIT
