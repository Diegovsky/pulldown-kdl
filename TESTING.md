# A background on the testing tool
Rust has bultin testing capabilities, but they have the following downsides:
  - Requires recompilation on every change.
  - Are slow to compile + execute (cycle).
  - Requires each test to be a function, which demands writing a lot of boilerplate.

Moreover, this project is unique in the way tests are done:
  - The correct behaviour is checked manually by a human, then, tests are generated from the fixtures.
  - Each time changes are made, the new implementation is checked against previous results.
    - If it passes, all is good.
    - If it doesn't, then:
      - A human must check if the failure is due to **unexpected behaviour** or if **the new behaviour is more correct**.
      - If it is wrong, they must fix it.
      - Otherwise, tests are re-generated based on the new behaviour.

This workflow maximises speed for testing and creating tests.

# How to use testman
Testman (`testman.py`) is a simple python script that works in various modes. Essentially, all it does it call the `tester` crate with multiple files.

It currently has 4 modes, 3 of which are implemented by the `tester` crate:
  - Check
  - Compare (default)
  - Emit
  - Extract (exclusive)

## Emit
This mode takes in each KDL file and emits an event stream into its corresponding `.json` file. If the KDL file failed to parse, it exits with an error.

This is used to both generate tests and separate those that this library can't parse yet.

## Check
This mode checks if the **file locations and token lengths** of the generated event stream from a `.json` file correspond to the source `.kdl` it was generated from.

This is used to check if the generated **file locations and token lengths** are correct for diagnostics, for example.

## Compare
This mode checks if each KDL file's generated event stream corresponds to the previously saved `.json` event stream.

This mode is used to check if the events emitted from the `.kdl` file match those of the `.json` file.

## Extract
This mode downloads a tarball from the `kdl` documentation definition and extracts it. Then, it filters all tests that passed (that is, could be parsed by `emit`) and replaces the `tests/` content with them.
