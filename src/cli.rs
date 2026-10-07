use std::{fmt, path::PathBuf};

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(version, about)]
pub(crate) struct Args {
    /// Source audio file.
    input: PathBuf,

    /// Language of the source recording.
    #[arg(long, value_enum)]
    language: Language,

    /// Optional Arabic dialect label.
    #[arg(long, value_enum)]
    dialect: Option<Dialect>,

    /// JSON formula; otherwise read formulas/default.json beside the executable.
    #[arg(long)]
    formula: Option<PathBuf>,

    /// Destination study audio file.
    #[arg(long)]
    output: PathBuf,

    /// Check the input path and formula without processing audio.
    #[arg(long)]
    check: bool,
}

#[derive(Clone, ValueEnum)]
enum Language {
    Ar,
    Ja,
    Es,
}

#[derive(Clone, ValueEnum)]
pub(crate) enum Dialect {
    Shami,
    Masri,
}

impl fmt::Display for Dialect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Shami => "shami",
            Self::Masri => "masri",
        })
    }
}

pub(crate) enum LanguageSetting {
    Arabic { dialect: Option<Dialect> },
    Japanese,
    Spanish,
}

impl fmt::Display for LanguageSetting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arabic {
                dialect: Some(dialect),
            } => write!(formatter, "ar ({dialect})"),
            Self::Arabic { dialect: None } => formatter.write_str("ar"),
            Self::Japanese => formatter.write_str("ja"),
            Self::Spanish => formatter.write_str("es"),
        }
    }
}

pub(crate) enum Mode {
    Check,
    Render,
}

pub(crate) struct Request {
    pub(crate) input: PathBuf,
    pub(crate) language: LanguageSetting,
    pub(crate) formula_path: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) mode: Mode,
}

impl TryFrom<Args> for Request {
    type Error = String;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let language = match (args.language, args.dialect) {
            (Language::Ar, dialect) => LanguageSetting::Arabic { dialect },
            (Language::Ja, None) => LanguageSetting::Japanese,
            (Language::Es, None) => LanguageSetting::Spanish,
            (_, Some(_)) => return Err("--dialect is only valid with --language ar".into()),
        };
        let formula_path = match args.formula {
            Some(path) => path,
            None => {
                let executable = std::env::current_exe()
                    .map_err(|error| format!("cannot locate executable: {error}"))?;
                executable
                    .parent()
                    .ok_or("executable has no parent directory")?
                    .join("formulas/default.json")
            }
        };

        Ok(Self {
            input: args.input,
            language,
            formula_path,
            output: args.output,
            mode: if args.check {
                Mode::Check
            } else {
                Mode::Render
            },
        })
    }
}
