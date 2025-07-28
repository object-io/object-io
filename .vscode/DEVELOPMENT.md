# ObjectIO Development Workspace

This VS Code workspace is configured for optimal development of the ObjectIO S3-compatible storage system.

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install SurrealDB
curl -sSf https://install.surrealdb.com | sh

# Install trunk for frontend development
cargo install trunk

# Install Redis (macOS)
brew install redis
```

### First-Time Setup

1. **Clone and Setup**:

   ```bash
   git clone <repository-url>
   cd object-io
   ```

2. **Install Dependencies**: VS Code will prompt to install recommended extensions

3. **Start Development Environment**:
   - Press `Cmd+Shift+P` → "Tasks: Run Task" → "start dev environment"
   - Or use the terminal tasks in VS Code

## 🛠️ Development Workflow

### Available VS Code Tasks

- **`cargo build`** - Build the project
- **`cargo test`** - Run all tests
- **`cargo clippy`** - Run linter
- **`full quality check`** - Run format, clippy, and tests
- **`start dev environment`** - Start both backend and frontend
- **`setup database`** - Start SurrealDB instance

### GitHub Copilot Integration

This workspace has custom Copilot instructions that understand:

- ObjectIO project structure and goals
- S3 API compatibility requirements
- Rust/Axum backend patterns
- Leptos frontend patterns
- Current sprint priorities and time estimates

### Code Snippets

Use these prefixes for quick code generation:

- `s3handler` - S3 API handler function
- `errortype` - Error type with thiserror
- `dbop` - Database operation function
- `leptoscomp` - Leptos component
- `test` - Async test function
- `config` - Configuration struct
- `apiresponse` - API response struct
- `validate` - Validation function

## 📋 Current Sprint Status

### Sprint 1: Core Foundation (6.5 days estimated)

- [ ] **CORE-001**: Project structure setup (1 day)
- [ ] **CORE-002**: SurrealDB setup (2 days)
- [ ] **CORE-003**: Axum HTTP server (1.5 days)
- [ ] **TEST-001**: Testing infrastructure (2 days)

**Priority**: Start with CORE-001 → CORE-002 → CORE-003

## 🎯 Development Guidelines

### Code Quality Standards

- **Format**: `cargo fmt` before commits
- **Lint**: `cargo clippy` must pass without warnings
- **Test**: Maintain >80% test coverage
- **Document**: Use `///` doc comments for public APIs

### Commit Message Format

```
feat(component): brief description

- Detailed change 1
- Detailed change 2

Fixes: OIO-XXX
```

### Architecture Principles

1. **S3 Compatibility**: Strict AWS S3 API compliance
2. **Performance**: Sub-100ms response times for object operations
3. **Security**: AWS SigV4 authentication, proper access controls
4. **Observability**: Comprehensive metrics and tracing
5. **Modularity**: Clean separation of concerns

### Time Management

- All issues have conservative time estimates
- Track actual vs estimated time daily
- Focus on MVP features first
- 25% buffer time included in Sprint 1

## 🔗 Quick Links

- **YouTrack Project**: [OIO Project Board](https://youtrack.devstroop.com/agiles/181-20)
- **Architecture Documentation**: See `architecture.md`
- **Sprint Planning**: [Time Estimates Article](https://youtrack.devstroop.com/articles/167-221)

## 🆘 Troubleshooting

### Common Issues

1. **SurrealDB Connection**: Ensure SurrealDB is running on localhost:8000
2. **Redis Connection**: Start Redis with `redis-server`
3. **Frontend Build**: Install trunk with `cargo install trunk`
4. **Extension Issues**: Restart VS Code after installing extensions

### Performance Tips

- Use `cargo build --release` for production builds
- Enable all features: `cargo build --all-features`
- Use `cargo clean` if builds become inconsistent

## 📝 Notes

- This workspace follows mydr24-docs standards
- Time estimates are deliberately conservative
- GitHub Copilot is configured with project-specific context
- All Sprint 1 issues must complete before Sprint 2 begins
