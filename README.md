# Rust CI Configuration

Taken from <https://github.com/jonhoo/rust-ci-conf>

To enable CI via this repo for a project:

```bash
git remote add ci https://github.com/biblius/rust-ci-config.git

git fetch ci

git merge --allow-unrelated ci/main
```

This will merge the workflows into your project.

Having a seperate repo for workflows enables us to update the workflows on one place and have the changes be reflected anywhere we decide to use them.
