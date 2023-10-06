# The Hextacy Book

To edit the book, use [mdbook](https://rust-lang.github.io/mdBook/).

If you don't have it, install it with

```bash
cargo install mdbook
```

Execute the following to generate the necessary files for github pages:

```bash
rm -rf docs/* && cd booklet && mdbook build && cd ../ && mv booklet/book/* booklet/book/.* docs/ && rm -r booklet/book
```
