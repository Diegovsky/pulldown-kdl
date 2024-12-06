# Pulldown-KDL
This project implements a pull parser for the [`KDL Document Language`] version 2, written in Rust.

Unlike node-based or serde-based parsers, pull parsers don't load the document at once in memory, choosing instead to "stream" events as it reads the document. This improves performance while also allowing other types of parsers to be built as higher level abstractions. Check out `pulldown-cmark`'s (a major inspiration in philosophy) short manifesto on [Why a pull parser](https://github.com/pulldown-cmark/pulldown-cmark?tab=readme-ov-file#why-a-pull-parser).


It's currently in its infancy, so, if you need a complete and stable library, take a look at [`kdl-rs`](https://github.com/kdl-org/kdl-rs/).

## What is implemented
  - [x] Nodes
    - [x] Inline nodes
    - [x] Without children
    - [x] With cildren
  - [x] Parameters
    - [x] Arguments
    - [x] Properties
  - [ ] Values 
    - [ ] String
      - [x] Ident String
      - [x] Quoted String
      - [ ] Raw Strings
      - [ ] Escapes
      - [ ] Multiline
    - [ ] Number
      - [ ] Keyword numbers (inf, -inf, nan)
      - [ ] Exponent
    - [ ] Boolean
    - [ ] Null
  - [ ] Comments
    - [ ] Inline
    - [ ] Multiline
    - [ ] Slashdash
  - [ ] Type Annotations
  - [ ] Other stuff
    - [x] Unicode
    - [ ] Line escapes

Since this crate is developed to serve [`htmeta`](https://github.com/Diegovsky/htmeta)'s purpose, features related to that project are prioritized, but PRs for other features are definitetely welcome!


[`KDL`]: https://kdl.dev
