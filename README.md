# git-mirror

![Rust][github-actions-badge]

**git-mirror** is a command-line program to keep local git mirrors up to date with their origin.

## Configuration

### Example configuration file
```toml
default_path = "/srv/git"

[[projects]]
url = "https://github.com/Bond-009/git-mirror.git"

[[projects]]
url = "https://git.zx2c4.com/cgit"
description = "A hyperfast web frontend for git repositories written in C."
```

### Example crontab entry
Execute git-mirror every 15 with the given config.
`*/15 * * * * git-mirror -c /srv/git/config.toml`

[github-actions-badge]: https://github.com/Bond-009/git-mirror/workflows/Rust/badge.svg
