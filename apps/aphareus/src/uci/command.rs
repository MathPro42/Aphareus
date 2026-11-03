use std::str::FromStr;

use engine::Color;
use search::Limits;

/// A line from the GUI, parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Uci,
    IsReady,
    UciNewGame,
    /// `setoption name <name> [value <value>]`: both may hold spaces.
    SetOption {
        name: String,
        value: Option<String>,
    },
    /// `position (startpos | fen <fen>) [moves <moves>]`, the moves still
    /// as text.
    Position {
        base: PositionBase,
        moves: Vec<String>,
    },
    /// `go`: `move_overhead` is left at 0, the caller sets it from the
    /// options.
    Go(Limits),
    Stop,
    Quit,
    /// `perft <depth>`: not standard, but common.
    Perft(u32),
    /// `d`: shows the board.
    Display,
    /// `eval`: shows the evaluation, term by term.
    Eval,
    /// `bench [depth]`: `Ok(None)` without a depth, `Err(text)` if the depth
    /// cannot be read.
    Bench(Result<Option<u32>, String>),
    /// An empty or unknown line: ignored, never an error.
    Unknown(String),
}

/// Where a `position` command starts from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PositionBase {
    StartPos,
    Fen(String),
}

/// Parses one line. Never fails: anything not understood is `Unknown`.
pub fn parse(line: &str) -> Command {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let Some((&first, rest)) = tokens.split_first() else {
        return Command::Unknown(line.to_string());
    };
    match first {
        "uci" => Command::Uci,
        "isready" => Command::IsReady,
        "ucinewgame" => Command::UciNewGame,
        "setoption" => parse_setoption(rest),
        "position" => parse_position(rest),
        "go" => Command::Go(parse_go(rest)),
        "stop" => Command::Stop,
        "quit" => Command::Quit,
        "perft" => Command::Perft(number(rest.first().copied()).unwrap_or(1)),
        "d" => Command::Display,
        "eval" => Command::Eval,
        "bench" => Command::Bench(parse_bench(rest.first().copied())),
        _ => Command::Unknown(line.to_string()),
    }
}

/// `name <name...> [value <value...>]`.
fn parse_setoption(tokens: &[&str]) -> Command {
    let after_name = match tokens.iter().position(|&t| t == "name") {
        Some(i) => &tokens[i + 1..],
        None => &[],
    };
    let value_at = after_name.iter().position(|&t| t == "value");
    let name = after_name[..value_at.unwrap_or(after_name.len())].join(" ");
    let value = value_at.map(|i| after_name[i + 1..].join(" "));
    Command::SetOption { name, value }
}

/// `(startpos | fen <fen...>) [moves <moves...>]`. The FEN runs up to
/// `moves`, so its optional counters may be missing.
fn parse_position(tokens: &[&str]) -> Command {
    let (head, moves) = match tokens.iter().position(|&t| t == "moves") {
        Some(i) => (&tokens[..i], &tokens[i + 1..]),
        None => (tokens, &[][..]),
    };
    let base = match head.split_first() {
        Some((&"fen", fen)) => PositionBase::Fen(fen.join(" ")),
        // `startpos`, or anything unclear: the starting position.
        _ => PositionBase::StartPos,
    };
    let moves = moves.iter().map(|&m| m.to_string()).collect();
    Command::Position { base, moves }
}

/// `go` limits, in any order. Other tokens (`ponder`, `searchmoves` and its
/// moves, `mate`…) are skipped one by one.
fn parse_go(tokens: &[&str]) -> Limits {
    let mut limits = Limits::default();
    let (white, black) = (Color::White.index(), Color::Black.index());
    let mut i = 0;
    while i < tokens.len() {
        let value = tokens.get(i + 1).copied();
        let mut has_value = true;
        match tokens[i] {
            "depth" => limits.depth = number(value),
            "movetime" => limits.movetime = millis(value),
            "wtime" => limits.time[white] = millis(value),
            "btime" => limits.time[black] = millis(value),
            "winc" => limits.inc[white] = millis(value),
            "binc" => limits.inc[black] = millis(value),
            "movestogo" => limits.movestogo = number(value),
            "nodes" => limits.nodes = number(value),
            "infinite" => {
                limits.infinite = true;
                has_value = false;
            }
            _ => has_value = false,
        }
        i += if has_value { 2 } else { 1 };
    }
    limits
}

/// `text` as a number, or `None` if missing or unreadable.
fn number<T: FromStr>(text: Option<&str>) -> Option<T> {
    text?.parse().ok()
}

/// The depth of `bench`, if any. Extra tokens are ignored.
fn parse_bench(depth: Option<&str>) -> Result<Option<u32>, String> {
    match depth {
        None => Ok(None),
        Some(text) => text.parse().map(Some).map_err(|_| text.to_string()),
    }
}

/// `text` as milliseconds. A negative time is 0.
fn millis(text: Option<&str>) -> Option<u64> {
    let ms: i64 = number(text)?;
    Some(ms.max(0).unsigned_abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    fn go(line: &str) -> Limits {
        match parse(line) {
            Command::Go(params) => params,
            other => panic!("{line:?} gave {other:?}"),
        }
    }

    fn ms(n: u64) -> Option<u64> {
        Some(n)
    }

    fn position(base: PositionBase, moves: &[&str]) -> Command {
        Command::Position {
            base,
            moves: moves.iter().map(|&m| m.to_string()).collect(),
        }
    }

    #[test]
    fn simple_commands() {
        assert_eq!(parse("uci"), Command::Uci);
        assert_eq!(parse("isready"), Command::IsReady);
        assert_eq!(parse("ucinewgame"), Command::UciNewGame);
        assert_eq!(parse("stop"), Command::Stop);
        assert_eq!(parse("quit"), Command::Quit);
        assert_eq!(parse("d"), Command::Display);
        assert_eq!(parse("eval"), Command::Eval);
    }

    #[test]
    fn position_startpos_with_moves() {
        assert_eq!(
            parse("position startpos moves e2e4 e7e5"),
            position(PositionBase::StartPos, &["e2e4", "e7e5"])
        );
        assert_eq!(
            parse("position startpos"),
            position(PositionBase::StartPos, &[])
        );
    }

    #[test]
    fn position_fen_with_moves() {
        assert_eq!(
            parse(&format!("position fen {START_FEN} moves e2e4")),
            position(PositionBase::Fen(START_FEN.to_string()), &["e2e4"])
        );
    }

    #[test]
    fn position_fen_without_counters() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";
        assert_eq!(
            parse(&format!("position fen {fen} moves e2e4 e7e5")),
            position(PositionBase::Fen(fen.to_string()), &["e2e4", "e7e5"])
        );
    }

    #[test]
    fn position_alone_or_unclear_is_startpos() {
        assert_eq!(parse("position"), position(PositionBase::StartPos, &[]));
        assert_eq!(
            parse("position moves e2e4"),
            position(PositionBase::StartPos, &["e2e4"])
        );
    }

    #[test]
    fn go_with_clocks() {
        let params = go("go wtime 300000 btime 300000 winc 0 binc 0");
        assert_eq!(
            params,
            Limits {
                time: [ms(300_000), ms(300_000)],
                inc: [ms(0), ms(0)],
                ..Limits::default()
            }
        );
    }

    #[test]
    fn go_single_limits() {
        assert_eq!(
            go("go infinite"),
            Limits {
                infinite: true,
                ..Limits::default()
            }
        );
        assert_eq!(
            go("go depth 5"),
            Limits {
                depth: Some(5),
                ..Limits::default()
            }
        );
        assert_eq!(
            go("go movetime 100"),
            Limits {
                movetime: ms(100),
                ..Limits::default()
            }
        );
        assert_eq!(
            go("go nodes 1000 movestogo 20"),
            Limits {
                nodes: Some(1000),
                movestogo: Some(20),
                ..Limits::default()
            }
        );
    }

    #[test]
    fn go_skips_what_it_does_not_know() {
        assert_eq!(
            go("go ponder searchmoves e2e4 d2d4 depth 3"),
            Limits {
                depth: Some(3),
                ..Limits::default()
            }
        );
        assert_eq!(
            go("go depth x movetime"),
            Limits::default(),
            "unreadable or missing values are ignored"
        );
    }

    #[test]
    fn go_negative_time_is_zero() {
        assert_eq!(go("go wtime -150 btime 1000").time, [ms(0), ms(1000)]);
    }

    #[test]
    fn setoption_with_value() {
        assert_eq!(
            parse("setoption name Hash value 128"),
            Command::SetOption {
                name: "Hash".to_string(),
                value: Some("128".to_string())
            }
        );
        assert_eq!(
            parse("setoption name Move Overhead value 30"),
            Command::SetOption {
                name: "Move Overhead".to_string(),
                value: Some("30".to_string())
            }
        );
    }

    #[test]
    fn setoption_without_value() {
        assert_eq!(
            parse("setoption name Clear Hash"),
            Command::SetOption {
                name: "Clear Hash".to_string(),
                value: None
            }
        );
    }

    #[test]
    fn perft_depth() {
        assert_eq!(parse("perft 4"), Command::Perft(4));
        assert_eq!(parse("perft"), Command::Perft(1));
        assert_eq!(parse("perft x"), Command::Perft(1));
    }

    #[test]
    fn bench_depth() {
        assert_eq!(parse("bench"), Command::Bench(Ok(None)));
        assert_eq!(parse("bench 3"), Command::Bench(Ok(Some(3))));
        assert_eq!(parse("bench 3 x"), Command::Bench(Ok(Some(3))));
    }

    #[test]
    fn bench_unreadable_depth_is_kept_as_text() {
        for depth in ["x", "-1", "2x"] {
            assert_eq!(
                parse(&format!("bench {depth}")),
                Command::Bench(Err(depth.to_string()))
            );
        }
    }

    #[test]
    fn unknown_lines() {
        for line in ["", "   ", "\t\n", "foo bar", "UCI", "goo depth 3"] {
            assert_eq!(parse(line), Command::Unknown(line.to_string()), "{line:?}");
        }
    }

    #[test]
    fn extra_spaces_are_ignored() {
        assert_eq!(
            parse("  position   startpos  moves  e2e4 \r"),
            position(PositionBase::StartPos, &["e2e4"])
        );
        assert_eq!(parse("isready\r"), Command::IsReady);
    }
}
