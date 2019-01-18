# simpleinfra

This repository contains various tools and scripts made by the Rust
Infrastructure Team to manage the project infrastructure.

## Webhooks Synchronization

The `sync-webhooks` tool synchronizes the webhooks configured in our
repositories with the configuration files located in `config/repo-webhooks`.
You can run it with:

```
$ cargo run -p sync-webhooks
```

You can also execute dry runs to check what changes the tool will make without
applying them:

```
$ cargo run -p sync-webhooks -- --dry
```

## restart-rcs

* `.ssh/config` should be configured correctly for all servers the
  scripts may log into
* rcs data/secrets should be in `/opt/rcs/data` on the target machine
