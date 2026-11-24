# Advent of Code 2026

My solutions to [Advent of Code 2026](https://adventofcode.com/2026) in Rust, with a writeup for each day typeset in [Typst](https://typst.app).

This year has 12 puzzles, one a day from 1 to 12 December. Every day has two parts, tests that check them on the puzzle's example, and a short report on how the solution works, with its code, answers and times.

## Requirements

- Rust 1.88 or newer (edition 2024).
- For the writeups: Typst 0.15 and the `craken-report` template installed as a local package, `@local/craken-report:0.1.0`, with its fonts. The template's `install.sh` sets up both.

## A day, start to finish

```sh
cargo scaffold 1          # creates the day's code, example, input and writeup files
```

1. Paste the example from the puzzle into `data/examples/01.txt`. If part two has a different example, save it as `data/examples/01-2.txt`.
2. Paste your input into `data/inputs/01.txt`.
3. Write `part_one` in `src/bin/01.rs`, and in its test swap `None` for the answer the puzzle gives for the example.
4. `cargo test --bin 01` until it passes, then `cargo solve 1` for the real answer.
5. Do the same for `part_two`.
6. Fill in `writeups/01.typ` and build it with `cargo writeup 1`.

## Commands

| Command | What it does |
| --- | --- |
| `cargo scaffold <day>` | Creates `src/bin/DD.rs`, `data/examples/DD.txt`, `data/inputs/DD.txt`, `writeups/DD.typ` and `writeups/results/DD.json`. It never overwrites a file that exists. |
| `cargo test` | Tests every day on its example. |
| `cargo test --bin <DD>` | Tests one day, e.g. `cargo test --bin 01`. |
| `cargo solve <day>` | Runs both parts on your input in a release build, and saves the answers and times for the writeup. |
| `cargo time <day\|all>` | The same, but reruns the day for about a second and keeps each part's median time. |
| `cargo all` | Runs every day so far, then prints a table of answers and times. A day that doesn't compile doesn't stop the others. |
| `cargo writeup <day\|all>` | Builds `writeups/DD.typ` into `writeups/pdf/day-DD.pdf`. Add `--watch` to rebuild on every save. |

These are aliases from `.cargo/config.toml` for `cargo run --bin aoc -- <command>`.

## Writing a solution

Every day is a standalone file with no shared code. `cargo scaffold` starts it like this:

```rust
pub fn part_one(_input: &str) -> Option<u64> {
    None
}

pub fn part_two(_input: &str) -> Option<u64> {
    None
}

// ── Runner ──
fn main() { … }            // runs both parts on data/inputs/01.txt and times them

#[cfg(test)]
mod tests {
    #[test]
    fn part_one_example() {
        assert_eq!(part_one(&example(1)), None);   // → Some(11)
    }
    // …
    fn example(part: u8) -> String { … }   // data/examples/01-2.txt for part two if it exists, else 01.txt
}
```

Write your code above the runner. A part returns `None` until it's solved. The answer can be any type that implements `Display`, such as `u64`, `i64`, `usize` or `String`, so change the return type to suit the puzzle. Keep the answer on one line.

`main` prints one line per part, like `part 1 (2314131 ns): 12345`, and `cargo solve` reads those lines. Leave that format alone. Anything else you print, such as debug output, is passed through.

## Writeups

Each `writeups/DD.typ` is a report in the Captain Craken house style. It has a cover with the day and a link to the puzzle, then sections for the puzzle, each part, the results, and what I learned. Notes in coral italics are placeholders to replace.

`writeups/lib.typ` adds a few helpers to the template:

| You write | You get |
| --- | --- |
| `#show: writeup.with(day: 1, title: […], subtitle: […])` | The cover and page setup. Any other `report` setting passes through, e.g. `contents: true`. |
| `#item(1, "part_one")` | A listing of `fn part_one` from `src/bin/01.rs`, with its doc comments and the file's line numbers. It also finds structs, enums, impls and constants. Add a label like `<lst:one>` to refer to it. |
| `#solution(1)` | The solution file up to the runner, so without `main` and the tests (`all: true` shows the whole file). |
| `#results(1)` | A table of both answers and their times, from `writeups/results/01.json`. |
| `#todo[…]` | A note to yourself, in coral, so it stands out in the PDF. |

The author's name and the year are set once, at the top of `writeups/lib.typ`.

The writeups read the Rust sources, so Typst has to treat the repository as the project root. `cargo writeup` handles that. To build a writeup by hand, or to preview it in an editor:

```sh
typst compile --root . writeups/01.typ writeups/pdf/day-01.pdf
typst compile --root . --input theme=lantern writeups/01.typ day-01-dark.pdf   # dark, for screens
```

In VS Code or Neovim with Tinymist, set its `rootPath` setting to the repository so the preview can find the sources.

The times in a writeup come from the last `cargo solve` or `cargo time`, so run one of them before the final build.

## Layout

```
src/
  main.rs          the scaffold / solve / time / all / writeup commands
  bin/DD.rs        one standalone file per day
data/
  examples/DD.txt  each puzzle's example (DD-2.txt when part two has its own)
  inputs/DD.txt    my puzzle inputs (not committed)
templates/         what cargo scaffold copies for each new day
writeups/
  lib.typ          helpers shared by every writeup
  DD.typ           one writeup per day
  results/DD.json  answers and times, written by cargo solve and cargo time
  pdf/             built writeups (not committed)
```

## Inputs

Advent of Code asks people not to share their puzzle inputs, so `data/inputs/` is in `.gitignore`. The examples are committed because the tests need them. To keep those private too, add `/data/examples/*` to `.gitignore`.

## License

The code is under the [MIT License](LICENSE). The puzzles belong to [Advent of Code](https://adventofcode.com/2026/about).
