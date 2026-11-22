use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs},
    Frame,
};
use scraper::Html;
use std::{env, error::Error};

const PAGE_URL: &str = "https://www.svt.se/text-tv/358";

#[derive(Debug)]
struct Team {
    rank: u32,
    name: String,
    played: u32,
    wins: u32,
    losses: u32,
    overtime_wins: u32,
    overtime_losses: u32,
    points: u32,
    goal_diff: i32,
}

fn parse_goals(value: &str) -> bool {
    let Some((for_goals, against_goals)) = value.split_once('-') else {
        return false;
    };
    for_goals.parse::<u32>().is_ok() && against_goals.parse::<u32>().is_ok()
}

fn parse_standings(text: &str) -> Vec<Team> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let start = tokens
        .iter()
        .position(|token| token.eq_ignore_ascii_case("ISHOCKEY"))
        .map(|index| index + 1)
        .unwrap_or(0);
    let mut teams = Vec::new();
    let mut index = start;

    while index + 8 < tokens.len() {
        let Ok(rank) = tokens[index].parse::<u32>() else {
            index += 1;
            continue;
        };
        let (
            Some(played),
            Some(wins),
            Some(losses),
            Some(overtime_wins),
            Some(overtime_losses),
            Some(points),
        ) = (
            tokens[index + 2].parse::<u32>().ok(),
            tokens[index + 3].parse::<u32>().ok(),
            tokens[index + 4].parse::<u32>().ok(),
            tokens[index + 5].parse::<u32>().ok(),
            tokens[index + 6].parse::<u32>().ok(),
            tokens[index + 8].parse::<u32>().ok(),
        )
        else {
            index += 1;
            continue;
        };

        if (1..=14).contains(&rank) && parse_goals(tokens[index + 7]) {
            let (goals_for, goals_against) = tokens[index + 7].split_once('-').unwrap();
            let goal_diff =
                goals_for.parse::<i32>().unwrap() - goals_against.parse::<i32>().unwrap();
            teams.push(Team {
                rank,
                name: tokens[index + 1].to_owned(),
                played,
                wins,
                losses,
                overtime_wins,
                overtime_losses,
                points,
                goal_diff,
            });
            index += 9;
        } else {
            index += 1;
        }
    }
    teams
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Original,
    Average,
}

fn render(frame: &mut Frame, teams: &[Team], view: View) {
    let areas = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .split(frame.area());

    let selected_tab = if view == View::Original { 0 } else { 1 };
    let tabs = Tabs::new([Line::from("Original tabell"), Line::from("Poängsnitt")])
        .select(selected_tab)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("SHL · SVT Text-TV 358"),
        );
    frame.render_widget(tabs, areas[0]);

    let mut ordered: Vec<&Team> = teams.iter().collect();
    if view == View::Original {
        ordered.sort_by_key(|team| team.rank);
    } else {
        ordered.sort_by(|a, b| {
            let avg_a = if a.played > 0 {
                a.points as f64 / a.played as f64
            } else {
                0.0
            };
            let avg_b = if b.played > 0 {
                b.points as f64 / b.played as f64
            } else {
                0.0
            };
            avg_b.total_cmp(&avg_a).then_with(|| a.name.cmp(&b.name))
        });
    }

    let mut rows = Vec::new();
    for (index, team) in ordered.iter().enumerate() {
        let average = if team.played > 0 {
            team.points as f64 / team.played as f64
        } else {
            0.0
        };
        let position = index + 1;
        rows.push(Row::new(vec![
            position.to_string(),
            team.name.clone(),
            team.played.to_string(),
            team.wins.to_string(),
            team.losses.to_string(),
            team.overtime_wins.to_string(),
            team.overtime_losses.to_string(),
            format!("{:+}", team.goal_diff),
            team.points.to_string(),
            format!("{average:.3}"),
        ]));
        if matches!(position, 6 | 10 | 12) {
            rows.push(Row::new(vec!["────────"; 10]).style(Style::default().fg(Color::DarkGray)));
        }
    }

    let header = Row::new([
        "Plats", "Lag", "M", "V", "F", "VÖ", "FÖ", "MS", "P", "P/match",
    ])
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);
    let widths = [
        Constraint::Length(5),
        Constraint::Min(16),
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(8),
    ];
    let title = if view == View::Original {
        "Grundserie"
    } else {
        "Sorterad på poängsnitt"
    };
    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1)
        .block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(table, areas[1]);

    let help = Paragraph::new(
        "←/→ eller Tab: växla tabell   O: original   S: poängsnitt   Q/Esc: avsluta",
    )
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, areas[2]);
}

fn run_tui(teams: &[Team]) -> Result<(), Box<dyn Error>> {
    let mut view = View::Original;
    ratatui::run(|terminal| loop {
        terminal.draw(|frame| render(frame, teams, view))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok::<(), std::io::Error>(()),
                KeyCode::Tab | KeyCode::Right | KeyCode::Left => {
                    view = if view == View::Original {
                        View::Average
                    } else {
                        View::Original
                    };
                }
                KeyCode::Char('o') => view = View::Original,
                KeyCode::Char('s') => view = View::Average,
                _ => {}
            }
        }
    })?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(argument) = env::args().nth(1) {
        return Err(format!(
            "Säsongsargument stöds inte ({argument}). SVT-sidan visar aktuell tabell."
        )
        .into());
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .build()?;
    let response = client.get(PAGE_URL).send()?.error_for_status()?;
    let html = response.text()?;
    if html.trim().is_empty() {
        return Err("SVT Text-TV svarade utan HTML-innehåll.".into());
    }

    let document = Html::parse_document(&html);
    let page_text = document.root_element().text().collect::<Vec<_>>().join(" ");
    let teams = parse_standings(&page_text);
    if teams.is_empty() {
        return Err(
            "Kunde inte hitta lagraderna på SVT Text-TV sida 358. Sidans format kan ha ändrats."
                .into(),
        );
    }
    run_tui(&teams)
}
