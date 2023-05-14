![Workflow badge](https://github.com/brunojppb/mailbolt/actions/workflows/ci.yml/badge.svg?branch=main)

# Mailbolt

A email subscription service built following along the [Zero to Production in Rust](https://www.zero2prod.com/index.html?country=Austria&discount_code=VAT20) book.

### Known issue on MacOS

While developing the app, more specifically when reaching chapter 7, I ran into a panic
with the message `Too many open files`. This is due to spawning too many app instances
for each test runs which bootstrap an entire application, database and socket connections.

These tests usually hit a limit that can be bumped with with the following command:

```shell
ulimit -n 8192
```
