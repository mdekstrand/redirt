# Recursive Directory Tool

`redirt`, the Recursive Directory Tool, is a small CLI tool for fast, efficient
listing, comparison, and copying of directory trees.  A little like `rsync`, but
less capable and optimized for local operation.


It uses [`ignore`][ignore] to walk file systems, and thus has similar walking and
exclusion behavior as [`ripgrep`][rg].

[rg]: https://github.com/BurntSushi/ripgrep
[ignore]: https://docs.rs/ignore/
