use std::fmt;
use std::num::IntErrorKind;

/// A `spin` option: an integer between `min` and `max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spin {
    pub name: &'static str,
    pub min: i64,
    pub default: i64,
    pub max: i64,
}

/// The `option` line that declares it in the `uci` answer.
impl fmt::Display for Spin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Spin {
            name,
            min,
            default,
            max,
        } = self;
        write!(
            f,
            "option name {name} type spin default {default} min {min} max {max}"
        )
    }
}

const HASH: Spin = Spin {
    name: "Hash",
    min: 1,
    default: 16,
    max: 33_554_432,
};

const THREADS: Spin = Spin {
    name: "Threads",
    min: 1,
    default: 1,
    max: 1024,
};

const MOVE_OVERHEAD: Spin = Spin {
    name: "Move Overhead",
    min: 0,
    default: 10,
    max: 5000,
};

/// Every option, in the order `uci` declares them.
pub const DECLARATIONS: [Spin; 3] = [HASH, THREADS, MOVE_OVERHEAD];

/// The values set by the GUI. None has an effect yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// Hash table size, in MB. Used from phase 5.
    pub hash: i64,
    /// Search threads. Used from phase 9.
    pub threads: i64,
    /// Time kept back on each move for lag, in ms. Used from phase 4.
    pub move_overhead: i64,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            hash: HASH.default,
            threads: THREADS.default,
            move_overhead: MOVE_OVERHEAD.default,
        }
    }
}

impl Options {
    /// Sets option `name` (any case) to `value`, clamped to its bounds.
    /// On an error, nothing changes.
    pub fn set(&mut self, name: &str, value: Option<&str>) -> Result<(), &'static str> {
        let (spin, field) = [
            (HASH, &mut self.hash),
            (THREADS, &mut self.threads),
            (MOVE_OVERHEAD, &mut self.move_overhead),
        ]
        .into_iter()
        .find(|(spin, _)| spin.name.eq_ignore_ascii_case(name))
        .ok_or("unknown option")?;
        // A number too big for `i64` is still a number: it gets clamped too.
        let value = match value.ok_or("missing value")?.parse::<i64>() {
            Ok(value) => value,
            Err(e) if *e.kind() == IntErrorKind::PosOverflow => i64::MAX,
            Err(e) if *e.kind() == IntErrorKind::NegOverflow => i64::MIN,
            Err(_) => return Err("not a number"),
        };
        *field = value.clamp(spin.min, spin.max);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(name: &str, value: &str) -> Options {
        let mut options = Options::default();
        options.set(name, Some(value)).unwrap();
        options
    }

    #[test]
    fn defaults() {
        let options = Options::default();
        assert_eq!(options.hash, 16);
        assert_eq!(options.threads, 1);
        assert_eq!(options.move_overhead, 10);
    }

    #[test]
    fn set_changes_the_right_field() {
        let default = Options::default();
        assert_eq!(
            set("Hash", "128"),
            Options {
                hash: 128,
                ..default
            }
        );
        assert_eq!(
            set("Threads", "4"),
            Options {
                threads: 4,
                ..default
            }
        );
        assert_eq!(
            set("Move Overhead", "50"),
            Options {
                move_overhead: 50,
                ..default
            }
        );
    }

    #[test]
    fn names_ignore_case() {
        assert_eq!(set("hash", "64").hash, 64);
        assert_eq!(set("MOVE OVERHEAD", "20").move_overhead, 20);
    }

    #[test]
    fn values_out_of_bounds_are_clamped() {
        assert_eq!(set("Hash", "0").hash, 1);
        assert_eq!(set("Hash", &i64::MAX.to_string()).hash, 33_554_432);
        assert_eq!(set("Threads", "-3").threads, 1);
        assert_eq!(set("Threads", "5000").threads, 1024);
        assert_eq!(set("Move Overhead", "-1").move_overhead, 0);
        assert_eq!(set("Move Overhead", "10000").move_overhead, 5000);
    }

    #[test]
    fn values_beyond_i64_are_clamped() {
        assert_eq!(set("Hash", "99999999999999999999").hash, 33_554_432);
        assert_eq!(set("Threads", "-99999999999999999999").threads, 1);
    }

    #[test]
    fn errors_change_nothing() {
        for (name, value, error) in [
            ("UCI_Foo", Some("1"), "unknown option"),
            ("Clear Hash", None, "unknown option"),
            ("Hash", None, "missing value"),
            ("Hash", Some("abc"), "not a number"),
            ("Threads", Some("1 2"), "not a number"),
        ] {
            let mut options = Options::default();
            assert_eq!(options.set(name, value), Err(error), "{name} {value:?}");
            assert_eq!(options, Options::default(), "{name} {value:?}");
        }
    }

    #[test]
    fn display_is_the_uci_declaration() {
        assert_eq!(
            MOVE_OVERHEAD.to_string(),
            "option name Move Overhead type spin default 10 min 0 max 5000"
        );
    }

    #[test]
    fn declarations() {
        let declared: Vec<_> = DECLARATIONS
            .iter()
            .map(|s| (s.name, s.min, s.default, s.max))
            .collect();
        assert_eq!(
            declared,
            [
                ("Hash", 1, 16, 33_554_432),
                ("Threads", 1, 1, 1024),
                ("Move Overhead", 0, 10, 5000),
            ]
        );
    }
}
