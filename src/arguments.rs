// SPDX-License-Identifier: GPL-2.0-only

//! OS argument tokens to informational output or a validated run configuration.
//!
//! Owns argument grammar, graph ceilings and Cargo-authoritative version output.
//! Must not initialize clocks, discover interfaces or touch terminal state.

use std::ffi::OsString;
use std::fmt;
use std::os::unix::ffi::OsStrExt;

pub(crate) const DEFAULT_CEILING_KBIT: u64 = 10240;
pub(crate) const DEFAULT_CEILING_BYTES: u64 = DEFAULT_CEILING_KBIT * 128;

/// Positive bytes/s ceilings shared by every startup interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ceilings {
    pub(crate) incoming: u64,
    pub(crate) outgoing: u64,
}

impl Default for Ceilings {
    fn default() -> Self {
        Self {
            incoming: DEFAULT_CEILING_BYTES,
            outgoing: DEFAULT_CEILING_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunConfig {
    /// Original bytes, never a displayed/escaped or lossy name.
    pub(crate) interface: Option<Vec<u8>>,
    pub(crate) ceilings: Ceilings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Invocation {
    Help,
    Version,
    Run(RunConfig),
}

/// The application writes this diagnostic to stderr and returns status 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArgumentError(&'static str);

impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for ArgumentError {}

/// Parse `std::env::args_os().skip(1)` (the executable token is excluded).
/// This pure operation never reads configuration, clocks, proc or terminals.
pub(crate) fn parse(
    tokens: impl IntoIterator<Item = OsString>,
) -> Result<Invocation, ArgumentError> {
    let tokens: Vec<_> = tokens.into_iter().collect();
    if tokens.len() == 1 {
        match tokens[0].as_bytes() {
            b"--help" => return Ok(Invocation::Help),
            b"--version" => return Ok(Invocation::Version),
            _ => {}
        }
    }

    let mut config = RunConfig {
        interface: None,
        ceilings: Ceilings::default(),
    };
    let mut options = true;
    let mut tokens = tokens.iter();
    while let Some(token) = tokens.next() {
        let bytes = token.as_bytes();
        if options {
            match bytes {
                b"--" => {
                    options = false;
                    continue;
                }
                b"-i" | b"-o" => {
                    let value = tokens
                        .next()
                        .ok_or(ArgumentError("ceiling option requires a separate value"))?;
                    let ceiling = parse_ceiling(value.as_bytes())?;
                    if bytes == b"-i" {
                        config.ceilings.incoming = ceiling;
                    } else {
                        config.ceilings.outgoing = ceiling;
                    }
                    continue;
                }
                b"--help" | b"--version" => {
                    return Err(ArgumentError("--help and --version must be standalone"));
                }
                _ if bytes.starts_with(b"-") => {
                    return Err(ArgumentError(
                        "unknown option; use -- before a leading-dash name",
                    ));
                }
                _ => {}
            }
        }
        if config.interface.is_some() {
            return Err(ArgumentError("at most one interface name is allowed"));
        }
        config.interface = Some(bytes.to_vec());
    }
    Ok(Invocation::Run(config))
}

fn parse_ceiling(bytes: &[u8]) -> Result<u64, ArgumentError> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return Err(ArgumentError(
            "ceiling must be a whole token of ASCII decimal digits",
        ));
    }
    let value = bytes.iter().try_fold(0_u64, |value, digit| {
        value.checked_mul(10)?.checked_add(u64::from(digit - b'0'))
    });
    match value {
        Some(0) => Ok(DEFAULT_CEILING_BYTES),
        Some(value) => value
            .checked_mul(128)
            .ok_or(ArgumentError("ceiling is outside the supported u64 range")),
        None => Err(ArgumentError("ceiling is outside the supported u64 range")),
    }
}

pub(crate) fn version_text() -> String {
    format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

pub(crate) fn help_text() -> String {
    format!(
        "Usage: {name} [-i VALUE] [-o VALUE] [--] [INTERFACE]\n\
         \x20      {name} --help\n\
         \x20      {name} --version\n\n\
         Monitor one interface; arrows navigate the complete startup list.\n\
         INTERFACE selects an exact, case-sensitive name; -- ends option parsing.\n\
         If the requested name is not found, use the first naturally sorted interface.\n\
         -i VALUE  Incoming graph ceiling\n\
         -o VALUE  Outgoing graph ceiling\n\
         VALUE is ASCII decimal binary-scaled kBit/s (1 kBit/s = 1024 Bit/s\n\
         = 128 bytes/s), from 1 through 144115188075855871.\n\
         Each direction defaults independently to {default} kBit/s (10.00 MBit/s).\n\
         Zero restores that direction's default; repeated options use the last value.\n\
         Ceiling values must be separate tokens; options may precede or follow a name.\n\
         --help and --version are standalone; no short aliases are accepted.\n",
        name = env!("CARGO_PKG_NAME"),
        default = DEFAULT_CEILING_KBIT,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;

    fn invoke(tokens: &[&str]) -> Result<Invocation, ArgumentError> {
        parse(tokens.iter().map(OsString::from))
    }

    fn run(tokens: &[&str]) -> RunConfig {
        let Invocation::Run(config) = invoke(tokens).unwrap() else {
            panic!("expected run configuration");
        };
        config
    }

    #[test]
    fn informational_outcomes_and_cargo_text() {
        assert_eq!(invoke(&["--help"]), Ok(Invocation::Help));
        assert_eq!(invoke(&["--version"]), Ok(Invocation::Version));
        assert_eq!(
            version_text(),
            format!("nload-rs {}\n", env!("CARGO_PKG_VERSION"))
        );
        let help = help_text();
        for content in [
            "Usage: nload-rs",
            "INTERFACE",
            "-i VALUE",
            "-o VALUE",
            "--help",
            "--version",
            "1024 Bit/s",
            "128 bytes/s",
            "10240 kBit/s",
            "10.00 MBit/s",
            "Zero restores",
            "standalone",
            "separate tokens",
            "-- ends option parsing",
        ] {
            assert!(help.contains(content), "missing help content: {content}");
        }
    }

    #[test]
    fn rejects_unknown_flags_multiple_names_and_information_combinations() {
        for tokens in [
            vec!["--unknown"],
            vec!["-h"],
            vec!["-V"],
            vec!["-"],
            vec!["eth0", "eth1"],
            vec!["--help", "--version"],
            vec!["--help", "eth0"],
            vec!["eth0", "--version"],
            vec!["--help", "-i", "10"],
            vec!["-o", "10", "--version"],
            vec!["--help", "--"],
            vec!["--version", "--version"],
            vec!["--", "eth0", "eth1"],
        ] {
            let error = invoke(&tokens).expect_err(&format!("accepted {tokens:?}"));
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn preserves_os_name_bytes_and_option_terminator() {
        let name = vec![b'e', 0xff, b'2'];
        let Invocation::Run(config) = parse([OsString::from_vec(name.clone())]).unwrap() else {
            panic!("expected run");
        };
        assert_eq!(config.interface, Some(name));
        for name in ["-eth0", "--help", "--version", "-i", "--"] {
            assert_eq!(run(&["--", name]).interface, Some(name.as_bytes().to_vec()));
        }
        assert_eq!(run(&["ETH0"]).interface, Some(b"ETH0".to_vec()));
        assert_eq!(run(&["--"]).interface, None);
        assert_eq!(run(&[""]).interface, Some(vec![])); // selection validates existence
        assert!(invoke(&["--", "eth0", "-i", "1"]).is_err());
    }

    #[test]
    fn independent_defaults_overrides_repetitions_and_zero() {
        let default = run(&[]);
        assert_eq!(default.interface, None);
        assert_eq!(
            default.ceilings,
            Ceilings {
                incoming: 1_310_720,
                outgoing: 1_310_720
            }
        );
        let before = run(&["-i", "20480", "-o", "5120", "eth0"]);
        assert_eq!(before, run(&["eth0", "-i", "20480", "-o", "5120"]));
        assert_eq!(
            before.ceilings,
            Ceilings {
                incoming: 2_621_440,
                outgoing: 655_360
            }
        );
        assert_eq!(run(&["-i", "10", "-i", "00020"]).ceilings.incoming, 2560);
        assert_eq!(run(&["-o", "10", "-o", "20"]).ceilings.outgoing, 2560);
        assert_eq!(
            run(&["-i", "10", "-o", "20", "-i", "0000"]).ceilings,
            Ceilings {
                incoming: 1_310_720,
                outgoing: 2560
            }
        );
        assert_eq!(
            run(&["-i", "10", "-o", "20", "-o", "0"]).ceilings,
            Ceilings {
                incoming: 1280,
                outgoing: 1_310_720
            }
        );
    }

    #[test]
    fn ceiling_range_and_whole_ascii_digit_grammar() {
        for flag in ["-i", "-o"] {
            for (value, expected) in [
                ("1", 128),
                ("144115188075855871", 18_446_744_073_709_551_488),
            ] {
                let ceilings = run(&[flag, value]).ceilings;
                assert_eq!(
                    if flag == "-i" {
                        ceilings.incoming
                    } else {
                        ceilings.outgoing
                    },
                    expected
                );
            }
            for value in [
                "144115188075855872",
                "18446744073709551615",
                "18446744073709551616",
                "99999999999999999999999999999999999999999",
                "",
                "+1",
                "-1",
                "1.0",
                " 1",
                "1 ",
                "1\t",
                "1k",
                "１２",
                "١",
                "--help",
            ] {
                assert!(invoke(&[flag, value]).is_err(), "accepted {flag} {value:?}");
            }
            assert!(invoke(&[flag]).is_err());
        }
        for attached in ["-i20480", "-i=20480", "-o5120", "-o=5120"] {
            assert!(invoke(&[attached]).is_err());
        }
        let long_zero = "0".repeat(1000);
        assert_eq!(
            run(&["-i", &long_zero]).ceilings.incoming,
            DEFAULT_CEILING_BYTES
        );
        let long_one = format!("{long_zero}1");
        assert_eq!(run(&["-i", &long_one]).ceilings.incoming, 128);
        assert!(parse([OsString::from("-i"), OsString::from_vec(vec![0xff])]).is_err());
        // Every occurrence must validate even if a later value would replace it.
        assert!(invoke(&["-i", "bad", "-i", "1"]).is_err());
    }
}
