# Claude Code Instructions for Voiceboard

## Project Overview

Voiceboard is a virtual microphone mixer application that allows users to mix their real microphone audio with sound files and output to a virtual microphone device.

## Language Policy

**All code, comments, documentation, commit messages, and any text in the project must be written in English only.**

## Development Guidelines

### Code Style

- Follow Rust conventions (rustfmt, clippy)
- Follow Angular style guide
- Use meaningful variable and function names
- Keep functions small and focused

### Linting and Formatting

**Run these commands before each commit:**

- Rust: `cargo fmt --manifest-path src-tauri/Cargo.toml` and `cargo clippy --manifest-path src-tauri/Cargo.toml`
- Angular: `npm run lint` (if available) or ensure ESLint/Prettier are applied

### Commits and Git

- Use conventional commits format
- Examples: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`
- Write clear, concise commit messages in English
- Use GitFlow workflow

### Testing

**Only commit if all tests pass**

- Write tests for new features
- Run `cargo test --manifest-path src-tauri/Cargo.toml` for Rust tests
- Run `npm test` for Angular tests

### Code Coverage

To measure Rust code coverage:

```bash
# Install cargo-tarpaulin (one-time)
cargo install cargo-tarpaulin

# Run coverage report
cd src-tauri && cargo tarpaulin --out Stdout
```

**Coverage targets:**
- Domain layer: 100% (unit tests)
- Application layer: 80%+ (unit tests + mocks)
- Adapters: Hardware-dependent code may have lower coverage

## Key Files

- `ROADMAP.md` - Project roadmap and task tracking
- `README.md` - Project overview and setup instructions
- `src-tauri/src/application/audio_engine.rs` - Core audio processing
- `src-tauri/src/application/commands.rs` - Tauri IPC commands
