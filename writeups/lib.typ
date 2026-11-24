// Shared pieces for the Advent of Code 2026 writeups, on top of craken-report.
//
// The writeups read the Rust sources in src/bin/, so they compile with the
// repository as the Typst root. `cargo writeup <day>` does that for you; by hand:
//   typst compile --root . writeups/01.typ writeups/pdf/day-01.pdf
#import "@local/craken-report:0.1.0": *

#let author = "Pietro Agnoli"
#let year = 2026

/// Two digits: 1 → "01".
#let pad(day) = if day < 10 { "0" + str(day) } else { str(day) }

/// The cover and page setup for one day:
///   #show: writeup.with(day: 1, title: [Historian Hysteria])
/// Any other craken-report setting passes through: contents: true, date: auto, …
#let writeup(day: none, title: none, subtitle: none, ..args, body) = {
  let url = "https://adventofcode.com/" + str(year) + "/day/" + str(day)
  let settings = (
    course: "Advent of Code " + str(year),
    lab: "Day " + str(day),
    title: title,
    subtitle: subtitle,
    authors: (author,),
    fields: (([Puzzle], link(url, url.trim("https://", at: start))),),
    date: datetime(year: year, month: 12, day: day),
  ) + args.named()
  report(..settings, body)
}

/// A note to yourself that stands out in the PDF until you replace it.
#let todo(body) = text(fill: palette.coral, style: "italic", body)

// ── The solution's code ──────────────────────────────────────────────────────

/// The source of day `day`: src/bin/DD.rs.
#let source(day) = read("../src/bin/" + pad(day) + ".rs")

/// The top-level item `name` (a fn, struct, enum, impl, const, …) in `src`,
/// with the doc comments and attributes above it, and its first line number.
#let _find-item(src, name) = {
  let lines = src.split("\n")
  let head = regex(
    "^(pub(\\([a-z]+\\))? )?((const|async|unsafe) )*(fn|struct|enum|union|trait|type|const|static|impl|mod) "
      + name
      + "\\b",
  )
  let start = lines.position(l => l.match(head) != none)
  assert(start != none, message: "no top-level `" + name + "` in the source")
  let first = start
  while first > 0 and (lines.at(first - 1).starts-with("///") or lines.at(first - 1).starts-with("#[")) {
    first -= 1
  }
  // A one-line item ends where it starts; anything else at the first `}` in column 0.
  let last = if lines.at(start).trim().ends-with(";") { start } else {
    let offset = lines.slice(start).position(l => l.starts-with("}"))
    assert(offset != none, message: "`" + name + "` has no closing brace in column 0")
    start + offset
  }
  (lines.slice(first, last + 1).join("\n"), first + 1)
}

/// A numbered listing of one top-level item of the day's solution, with the
/// file's line numbers. Label it to refer to it: #item(day, "part_one") <lst:one>
#let item(day, name, caption: auto) = {
  let (code, first-line) = _find-item(source(day), name)
  listing(
    caption: if caption == auto { [#raw(name) in #raw("src/bin/" + pad(day) + ".rs").] } else { caption },
    lang: [Rust],
    numbers: true,
    first-line: first-line,
  )[#raw(code, lang: "rust", block: true)]
}

/// The solution file as a listing. It stops before the runner every day shares
/// (the comments above `fn main`, then `main` and the tests) unless `all: true`.
#let solution(day, all: false, caption: auto) = {
  let lines = source(day).split("\n")
  let cut = if all { none } else {
    lines.position(l => l.starts-with("fn main(") or l.starts-with("#[cfg(test)]"))
  }
  if cut != none {
    while cut > 0 and (lines.at(cut - 1).starts-with("//") or lines.at(cut - 1).trim() == "") { cut -= 1 }
  }
  let code = (if cut == none { lines } else { lines.slice(0, cut) }).join("\n").trim(at: end)
  listing(
    caption: if caption == auto { [The full solution, #raw("src/bin/" + pad(day) + ".rs").] } else { caption },
    lang: [Rust],
    numbers: true,
  )[#raw(code, lang: "rust", block: true)]
}

// ── Answers and times ────────────────────────────────────────────────────────

/// 1234567 → "1.23 ms"; matches what `cargo solve` prints.
#let format-time(ns) = {
  if ns == none { return [—] }
  let (value, unit, digits) = if ns < 1000 { (ns, "ns", 0) } else if ns < 1000000 {
    (ns / 1e3, "µs", 1)
  } else if ns < 1000000000 { (ns / 1e6, "ms", 2) } else { (ns / 1e9, "s", 2) }
  let s = str(calc.round(value, digits: digits))
  // calc.round drops trailing zeros: put them back so the column lines up.
  if digits > 0 {
    let decimals = if s.contains(".") { s.split(".").at(1).len() } else { 0 }
    if decimals == 0 { s += "." }
    s += "0" * (digits - decimals)
  }
  [#s #unit]
}

/// The day's answers and times, from writeups/results/DD.json. `cargo solve`
/// and `cargo time` write that file; `cargo scaffold` starts it empty.
#let results(day, caption: auto) = {
  let r = json("results/" + pad(day) + ".json")
  let answer(p) = if p.answer == none { text(fill: palette.ink-soft)[not solved yet] } else {
    text(number-width: "tabular", p.answer)
  }
  let benched = r.part1.runs > 1 or r.part2.runs > 1
  figure(
    table(
      columns: 3,
      align: (start, end, end),
      table.header[Part][Answer][Time],
      [One], answer(r.part1), format-time(r.part1.time_ns),
      [Two], answer(r.part2), format-time(r.part2.time_ns),
    ),
    caption: if caption != auto { caption } else if benched [
      Answers for my input. Times are the median of repeated runs of the release build (`cargo time`).
    ] else [
      Answers for my input, with the time of one run of the release build (`cargo solve`).
    ],
  )
}
