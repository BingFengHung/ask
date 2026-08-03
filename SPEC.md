# Technical Specification: `ask` Natural Language Shell CLI Assistant

## Problem Statement

As software engineers and power users, remembering complex terminal commands (such as `ffmpeg` video encoding, `docker` container cleanup, `git rebase` manipulations, or OS-specific network commands) requires frequent context switching to Google, StackOverflow, or browser-based AI chats. This context switching disrupts focus and reduces terminal productivity.

## Solution

`ask` is a lightweight, zero-runtime-dependency Rust CLI tool. Developers type natural language queries directly into their terminal (e.g. `ask "kill process running on port 8080"`). 

The tool detects the current OS environment, translates the query into an accurate shell command via a 3-tier LLM fallback strategy (`agy CLI` -> `Ollama` -> `OpenAI/DeepSeek API`), displays the command with syntax highlighting, and provides an interactive safety confirmation prompt (`[Y]es / [n]o / [e]dit`) before executing the command.

Cross-platform binaries (`.exe` for Windows, binaries for macOS and Linux) are automatically compiled and delivered via GitHub Actions CI/CD pipelines.

---

## User Stories

1. As a developer, I want to type `ask "<natural language text>"` in my terminal so that I can quickly generate shell commands without leaving my CLI.
2. As a Windows user, I want the tool to automatically recognize PowerShell syntax so that generated commands run natively on my OS.
3. As a macOS/Linux user, I want the tool to automatically recognize Zsh/Bash syntax so that generated commands are compatible with my shell environment.
4. As a user with `agy` CLI installed, I want `ask` to automatically leverage `agy` for LLM inference so that I don't need to configure extra API keys or start local servers manually.
5. As a privacy-focused developer, I want `ask` to connect to local `Ollama` models if `agy` is not available, so that my terminal queries remain entirely offline.
6. As a cloud AI user, I want `ask` to fall back to `OPENAI_API_KEY` or `DEEPSEEK_API_KEY` when no local LLMs are active, so that I always get reliable results.
7. As a safety-conscious engineer, I want `ask` to display the generated command in high-contrast text and ask for interactive confirmation (`[Y/n/e]`) before executing any destructive operations.
8. As a user who wants to tweak a command before execution, I want an `[e]dit` option in the confirmation prompt so that I can modify flags or arguments inline.
9. As a developer without a local Rust toolchain, I want compiled binaries (`.exe` and unix binaries) to be generated automatically via GitHub Actions releases so that I can install the tool in seconds.

---

## Implementation Decisions

### 1. Architecture & Core Modules (Rust Crate Structure)

The binary crate `ask` will be structured into 4 decoupled modules:

- **`main.rs`**: Entry point, CLI argument parsing via `clap`, and top-level workflow orchestration.
- **`env_detector.rs`**: Detects current OS (`Windows`, `MacOS`, `Linux`), default shell (`PowerShell`, `Zsh`, `Bash`), and current working directory.
- **`llm_provider.rs`**: Implements the 3-tier fallback strategy trait (`LlmProvider`):
  1. `AgyProvider`: Executes `agy -p "<prompt>"` in a subprocess.
  2. `OllamaProvider`: Sends HTTP POST request to `http://localhost:11434/api/generate` via `reqwest`.
  3. `CloudApiProvider`: Sends HTTP POST request to OpenAI / DeepSeek API endpoints if API keys are set.
- **`ui.rs`**: Renders colored output (`colored` crate) and handles interactive prompts (`inquire` or `dialoguer` crate).
- **`executor.rs`**: Spawns the current OS shell to execute the confirmed command and streams output to `stdout`/`stderr`.

### 2. LLM Prompting & System Instructions

The prompt sent to the LLM enforces strict output formatting:
- System Role: Expert Systems Administrator and Shell Command Generator.
- Context: Current OS, Shell Name, and Working Directory.
- Constraints: Return ONLY the exact raw shell command string without markdown code fences or conversational fluff.

### 3. CI/CD GitHub Actions Pipeline

A `.github/workflows/release.yml` workflow will be created:
- Triggers on git tags matching `v*` or pushes to `main`.
- Matrix targets:
  - `x86_64-pc-windows-msvc` (Windows `ask.exe`)
  - `x86_64-apple-darwin` & `aarch64-apple-darwin` (macOS Intel & Apple Silicon)
  - `x86_64-unknown-linux-gnu` (Linux)
- Uploads build artifacts to GitHub Actions Artifacts & GitHub Releases.

---

## Testing Decisions

### Test Strategy & Seams
- **External Behavior Testing**: Unit tests focus on CLI argument parsing, OS environment detection, and LLM prompt formatting.
- **Mock LLM Seams**: The `LlmProvider` trait allows mocking LLM responses in unit tests without making actual network or subprocess calls.
- **Dry-run Integration Tests**: Tests verify that selecting `[n]` in confirmation mode exits cleanly with code 0 without executing subprocess commands.

---

## Out of Scope

- Shell autocomplete plugins (zsh autosuggestions integration) - can be added in v2.
- Multi-step conversational agent memory across terminal commands (each query is single-turn stateless).
- Remote SSH execution.

---

## Further Notes

- Cargo Crates used: `clap` (v4), `reqwest` (v0.12, async/blocking), `tokio` (v1), `colored` (v2), `inquire` (v0.7), `serde` / `serde_json`.
