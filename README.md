<p align="center"><strong>offcodex</strong> is a local-first coding agent CLI, forked from Codex and optimized for small and medium local LLMs.</p>

<p align="center">
  <img src="https://github.com/leonardo-u/offcodex/blob/main/.github/codex-cli-splash.png" alt="offcodex splash" width="80%" />
</p>

---

## What is offcodex?

offcodex (Offline Codex) is a fork of Codex CLI designed to run coding agents primarily through local model providers. Its default provider is [Ollama](https://ollama.com/), with a default model of `qwen2.5:14b`.

It is tuned for models such as `qwen2.5-coder:14b`, `hhao/qwen2.5-coder-tools:14b`, and compatible Qwen or DeepSeek models. The local agent loop includes a compact tool surface, explicit tool-use instructions, conservative coding sampling settings, and a parser fallback for models that return tool calls as JSON text instead of native function calls.

Local models can inspect, create, edit, and patch files, and run terminal commands through the sandboxed tool runtime. The Linux startup check detects common Bubblewrap/user-namespace failures and offers approved repairs individually, either until reboot or permanently.

## Quickstart

### Prerequisites

- Rust toolchain and `cargo`
- Ollama running locally
- A local coding model, for example:

```shell
ollama pull qwen2.5:14b
ollama pull qwen2.5-coder:14b
```

### Install from this repository

Clone the fork and run its isolated installer:

```shell
git clone https://github.com/leonardo-u/offcodex.git
cd offcodex
./scripts/install/install-local.sh
```

This builds the project and installs only `~/.local/bin/offcodex`; it does not replace an existing `codex` binary.

Start the agent with:

```shell
offcodex
```

Use `/model` to select any model installed in Ollama, and `/auto on` or `/auto off` to change tool-approval behavior without restarting the session.

<details>
<summary>You can also go to the <a href="https://github.com/leonardo-u/offcodex/releases/latest">latest offcodex GitHub Release</a> and download the appropriate binary for your platform.</summary>

Release assets use the `offcodex-*` naming convention. If you download an archive directly, extract it and place the `offcodex` binary somewhere on your `PATH`.

</details>

## Local-model behavior

- Ollama is the default local provider; `--local-provider ollama` remains available for explicit use.
- Requests use conservative local coding options (`temperature = 0.1`, `num_ctx = 16384`).
- Tool calls are validated and malformed JSON is returned to the model with a correction request instead of crashing the session.
- For local models, offcodex exposes a small, reliable baseline of file, patch, terminal, and web tools rather than overwhelming the model with every available integration.
- Commands still run in the configured sandbox and retain normal approval controls.

## Docs

- [**Installation and build notes**](./docs/install.md)
- [**Contributing**](./docs/contributing.md)
- [**Issues and releases**](https://github.com/leonardo-u/offcodex)

This repository is licensed under the [Apache-2.0 License](LICENSE).
