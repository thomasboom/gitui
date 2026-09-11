# asyncjj

`asyncjj` provides non-blocking access to [Jujutsu (`jj`)](https://docs.jj-vcs.dev/latest/)
operations, enabling a TUI to perform potentially slow VCS operations in the
background while keeping the user interface responsive.

It also provides synchronous Jujutsu operations.

## Design

Unlike the former `asyncgit` crate (which linked `git2`/`gix` directly),
`asyncjj` shells out to the `jj` CLI:

* `jj-lib` is intentionally unstable (frequent breaking releases) and meant to
  be consumed by the `jj` binary itself — see
  https://docs.jj-vcs.dev/latest/technical/architecture/.
* The CLI is the stable automation surface: `--no-pager`, `--color never`,
  `-R <repo>`, machine-readable `-T` templates, and `--ignore-working-copy`
  for read-only queries.
* This mirrors how existing jj TUIs (e.g. `lazyjj`) interoperate with jj.

All jj invocations go through `sync::jj_cmd::run_jj`, which pins
`--no-pager --color never` and surfaces stderr on failure as
`Error::Jj { command, status, stderr }`.
