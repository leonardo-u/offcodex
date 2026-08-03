<p align="center"><strong>offcodex</strong> is a local-first coding agent CLI, forked from Codex and optimized for small and medium local LLMs.</p>

<p align="center">
  <img src="https://github.com/leonardo-u/offcodex/blob/main/.github/codex-cli-splash.png" alt="offcodex splash" width="80%" />
</p>

---

## What is offcodex?

offcodex (Offline Codex) is a fork of Codex CLI designed to run coding agents primarily through local model providers. Its default provider is [Ollama](https://ollama.com/), with a default model of `qwen2.5:14b`.

It is tuned for models such as `qwen2.5-coder:14b`, `hhao/qwen2.5-coder-tools:14b`, and compatible Qwen or DeepSeek models. The local agent loop includes a compact tool surface, explicit tool-use instructions, conservative coding sampling settings, and a parser fallback for models that return tool calls as explicitly tagged JSON text instead of native function calls.

Local models can inspect, create, edit, and patch files, and run terminal commands through the sandboxed tool runtime. The Linux startup check detects common Bubblewrap/user-namespace failures and offers approved repairs individually, either until reboot or permanently.

### Linux sandbox recovery

offcodex runs a short Bubblewrap self-test at startup when the Linux tool sandbox is enabled. This catches failures before a local model attempts a command and then incorrectly assumes that a file was created or a command was executed.

When the test fails, offcodex checks for common host-side causes and presents each applicable repair separately:

- unprivileged user namespaces disabled by the kernel;
- a zero `user.max_user_namespaces` limit;
- Ubuntu/AppArmor restrictions that prevent Bubblewrap from mapping the user namespace.

Every repair requires explicit approval. After approving a repair, choose either **until the next reboot** (a global kernel setting that resets after reboot) or **permanently** (a dedicated `/etc/sysctl.d/` configuration file). These settings cannot be limited to one offcodex process, so “temporary” intentionally means “until reboot”, not merely “until this terminal closes”.

The repair uses PolicyKit (`pkexec`) and `sysctl`; if either is unavailable, authorization is denied, or a container/organization policy blocks the change, offcodex reports the command failure and gives recovery guidance. It does not silently weaken the sandbox or fall back to unsandboxed command execution.

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

Use `/model` to select any model installed in Ollama, and `/auto on` or `/auto off` to change tool-approval behavior without restarting the session. `/auto off` is the cautious default: untrusted tool actions require your approval.

<details>
<summary>You can also go to the <a href="https://github.com/leonardo-u/offcodex/releases/latest">latest offcodex GitHub Release</a> and download the appropriate binary for your platform.</summary>

Release assets use the `offcodex-*` naming convention. If you download an archive directly, extract it and place the `offcodex` binary somewhere on your `PATH`.

</details>

## Local-model behavior

- Ollama is the default local provider; `--local-provider ollama` remains available for explicit use.
- Requests use conservative local coding options (`temperature = 0.1`, `num_ctx = 16384`).
- Tool calls are validated and malformed JSON is returned to the model with a correction request instead of crashing the session.
- When `/model` switches an Ollama model, offcodex visibly reads its `/api/show` template and reports the explicit textual tool-call wrapper it declares. Native function calling is still preferred.
- The textual fallback accepts only explicit template wrappers such as `<tool_call>…</tool_call>`, `<function_call>…</function_call>`, and `<tools>…</tools>`. A bare JSON snippet in normal assistant prose is never executed as a command.
- For local models, offcodex exposes a small, reliable baseline of file, patch, terminal, and web tools rather than overwhelming the model with every available integration.
- Commands still run in the configured sandbox and retain normal approval controls; `/auto off` asks before untrusted mutations, while `/auto on` enables autonomous execution.

## Docs

- [**Installation and build notes**](./docs/install.md)
- [**Contributing**](./docs/contributing.md)
- [**Issues and releases**](https://github.com/leonardo-u/offcodex)

This repository is licensed under the [Apache-2.0 License](LICENSE).
