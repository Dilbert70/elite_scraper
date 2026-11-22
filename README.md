# SHL Table TUI

A small Rust terminal app that fetches the current SHL standings from [SVT Text-TV page 358](https://www.svt.se/text-tv/358) and displays them in an interactive Ratatui interface.

## Run

```sh
cargo run
```

The app needs an internet connection to fetch the latest table. It reads the current season; historical seasons are not supported.

## Views and controls

- **Original tabell** keeps the order shown by SVT Text-TV.
- **Poängsnitt** sorts teams by points per game, highest first. Points per game is calculated as `points / games played`.
- Press `Tab` or `←` / `→` to switch views.
- Press `O` for the original table or `S` for the points-per-game table.
- Press `Q` or `Esc` to quit.

Horizontal rules divide places 1–6, 7–10, 11–12, and 13–14. In the points-per-game view, these divisions follow that view's sorted order.

## Columns

| Column | Meaning |
| --- | --- |
| M | Games played |
| V | Wins |
| F | Losses |
| VÖ | Overtime wins |
| FÖ | Overtime losses |
| MS | Goal difference |
| P | Standings points |
| P/match | Points per game |

## Data source

The app scrapes the visible text on SVT Text-TV page 358. The source page and its format may change, which can require updating the parser.
