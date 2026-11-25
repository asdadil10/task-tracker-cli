# tt

A small command-line time tracker written in Rust. Tasks and their tracked time are stored in `tasks.json` in the current directory.

## Build

```sh
cargo build --release
```

The binary will be available at `target/release/tt`.

## Usage

```sh
tt <COMMAND> [ARGS]
```

| Command | Description |
| --- | --- |
| `add <TITLE> [--tag \| -t <TAG>]` | Add a new task. |
| `start <ID>` | Start tracking a task. Pauses any running task. |
| `stop` | Stop the running timer. |
| `done <ID>` | Mark a task as completed. |
| `list [--all \| -a \| --tag \| -t <TAG>]` | List tasks. Completed tasks are hidden by default. |
| `summary` | Show total tracked time grouped by tag. |
| `delete <ID>` | Delete a task. |
| `<ID>` | Show details for one task. |
| `help [COMMAND]` | Show help. |
| `-v`, `--version` | Show the version. |

Examples:

```sh
tt add "Fix login bug" --tag backend
tt start 1
tt list --tag backend
tt summary
tt done 1
```
