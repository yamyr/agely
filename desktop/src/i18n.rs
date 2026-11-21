//! Native localization uses the same authored messages and formatting conventions as the website.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use regex::{Captures, Regex};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;
use time::OffsetDateTime;
use unicode_normalization::UnicodeNormalization;

use crate::agent::{AgentBrief, Locale};

#[derive(Clone, Deserialize)]
pub struct Language {
    pub id: String,
    pub label: String,
    pub intl: String,
    digits: Vec<String>,
    grouping: Grouping,
    number: NumberPattern,
    money: NumberPattern,
    date_pattern: Vec<DatePart>,
    months: Vec<String>,
    weekdays: Vec<String>,
    hour_cycle: String,
    hour_two_digits: bool,
    day_periods: Vec<String>,
    list_pair: String,
    list_three: String,
}

#[derive(Clone, Deserialize)]
struct Grouping {
    separator: String,
    primary: usize,
    secondary: usize,
    minimum_digits: usize,
}

#[derive(Clone, Deserialize)]
struct Affixes {
    prefix: String,
    suffix: String,
}

#[derive(Clone, Deserialize)]
struct NumberPattern {
    positive: Affixes,
    negative: Affixes,
}

#[derive(Clone, Deserialize)]
struct DatePart {
    kind: String,
    value: String,
}

struct Assets {
    languages: Vec<Language>,
    catalogs: BTreeMap<String, BTreeMap<String, String>>,
    content: Value,
    live_briefs: BTreeMap<String, AgentBrief>,
    slots: Regex,
}

#[derive(Clone)]
pub struct I18n {
    assets: Arc<Assets>,
    language: Language,
}

impl I18n {
    pub fn new(locale: &str) -> Result<Self> {
        let assets = Arc::new(Assets {
            languages: serde_json::from_str(include_str!("../assets/formatting.json"))
                .context("Could not read the bundled language formatting")?,
            catalogs: serde_json::from_str(include_str!("../assets/catalogs.json"))
                .context("Could not read the bundled translations")?,
            content: serde_json::from_str(include_str!("../assets/content.json"))
                .context("Could not read the bundled walkthrough")?,
            live_briefs: serde_json::from_str(include_str!("../assets/live_briefs.json"))
                .context("Could not read the bundled conversation instructions")?,
            slots: Regex::new(r"\{[0-9]+\}")?,
        });
        let id = resolve_locale(locale);
        let language = assets
            .languages
            .iter()
            .find(|language| language.id == id)
            .context("The bundled language registry has no English fallback")?
            .clone();
        Ok(Self { assets, language })
    }

    pub fn locale(&self) -> &str {
        &self.language.id
    }

    pub fn locales(&self) -> &[Language] {
        &self.assets.languages
    }

    /// A language change starts fresh domain state; callers confirm before using the returned seed.
    pub fn with_locale(&self, locale: &str) -> Self {
        let id = resolve_locale(locale);
        let language = self
            .assets
            .languages
            .iter()
            .find(|language| language.id == id)
            .cloned()
            .unwrap_or_else(|| self.language.clone());
        Self {
            assets: Arc::clone(&self.assets),
            language,
        }
    }

    pub fn t(&self, source: &str, values: &[String]) -> String {
        let candidate = self
            .assets
            .catalogs
            .get(self.locale())
            .and_then(|catalog| catalog.get(source));
        let message = match candidate {
            Some(candidate)
                if !candidate.trim().is_empty() && self.slots(candidate) == self.slots(source) =>
            {
                candidate
            }
            _ => source,
        };
        self.interpolate(message, values)
    }

    fn slots<'a>(&self, text: &'a str) -> BTreeSet<&'a str> {
        self.assets
            .slots
            .find_iter(text)
            .map(|slot| slot.as_str())
            .collect()
    }

    fn interpolate(&self, message: &str, values: &[String]) -> String {
        self.assets
            .slots
            .replace_all(message, |slot: &Captures<'_>| {
                let placeholder = slot.get(0).map(|part| part.as_str()).unwrap_or_default();
                let index = placeholder
                    .strip_prefix('{')
                    .and_then(|text| text.strip_suffix('}'))
                    .and_then(|text| text.parse::<usize>().ok());
                index
                    .and_then(|index| values.get(index))
                    .map(String::as_str)
                    .unwrap_or(placeholder)
                    .to_owned()
            })
            .into_owned()
    }

    pub fn number(&self, value: i64) -> String {
        self.format_integer(value, &self.language.number)
    }

    pub fn money(&self, value: i64) -> String {
        self.format_integer(value, &self.language.money)
    }

    fn format_integer(&self, value: i64, pattern: &NumberPattern) -> String {
        let digits = value.unsigned_abs().to_string();
        let grouping = &self.language.grouping;
        let mut groups = Vec::new();
        let mut end = digits.len();
        let mut size = grouping.primary;
        if end >= grouping.minimum_digits && size > 0 && grouping.secondary > 0 {
            while end > size {
                groups.push(&digits[end - size..end]);
                end -= size;
                size = grouping.secondary;
            }
        }
        groups.push(&digits[..end]);
        groups.reverse();
        let number = self.local_digits(&groups.join(&grouping.separator));
        let affixes = if value < 0 {
            &pattern.negative
        } else {
            &pattern.positive
        };
        format!("{}{}{}", affixes.prefix, number, affixes.suffix)
    }

    fn local_digits(&self, value: &str) -> String {
        value
            .chars()
            .map(|character| match character.to_digit(10) {
                Some(digit) => self
                    .language
                    .digits
                    .get(digit as usize)
                    .cloned()
                    .unwrap_or_else(|| character.to_string()),
                None => character.to_string(),
            })
            .collect()
    }

    pub fn session_time(&self, date: OffsetDateTime) -> String {
        let language = &self.language;
        let hour = match language.hour_cycle.as_str() {
            "h11" => date.hour() % 12,
            "h12" => match date.hour() % 12 {
                0 => 12,
                hour => hour,
            },
            "h24" => match date.hour() {
                0 => 24,
                hour => hour,
            },
            _ => date.hour(),
        };
        language
            .date_pattern
            .iter()
            .map(|part| match part.kind.as_str() {
                "literal" => part.value.clone(),
                "weekday" => language
                    .weekdays
                    .get(date.weekday().number_days_from_sunday() as usize)
                    .cloned()
                    .unwrap_or_default(),
                "month" => language
                    .months
                    .get(date.month() as usize - 1)
                    .cloned()
                    .unwrap_or_default(),
                "day" => self.local_digits(&date.day().to_string()),
                "year" => self.local_digits(&date.year().to_string()),
                "hour" => self.local_digits(&if language.hour_two_digits {
                    format!("{hour:02}")
                } else {
                    hour.to_string()
                }),
                "minute" => self.local_digits(&format!("{:02}", date.minute())),
                "dayPeriod" => language
                    .day_periods
                    .get(usize::from(date.hour() >= 12))
                    .cloned()
                    .unwrap_or_default(),
                _ => part.value.clone(),
            })
            .collect()
    }

    pub fn list(&self, values: &[String]) -> String {
        match values {
            [] => String::new(),
            [one] => one.clone(),
            [one, two] => self.interpolate(&self.language.list_pair, &[one.clone(), two.clone()]),
            [first, middle @ .., last] => {
                let separator = self
                    .language
                    .list_three
                    .split("{1}")
                    .next()
                    .and_then(|prefix| prefix.strip_prefix("{0}"))
                    .unwrap_or(", ");
                let beginning = std::iter::once(first.as_str())
                    .chain(middle.iter().map(String::as_str))
                    .collect::<Vec<_>>()
                    .join(separator);
                let ending = self
                    .language
                    .list_three
                    .split("{1}")
                    .nth(1)
                    .unwrap_or("{2}");
                format!(
                    "{beginning}{}",
                    self.interpolate(ending, &[String::new(), String::new(), last.clone()])
                )
            }
        }
    }

    pub fn localized_seed<T: DeserializeOwned>(&self) -> Result<T> {
        let mut seed = self.assets.content.clone();
        self.translate_seed(&mut seed, "", "");
        serde_json::from_value(seed)
            .context("The bundled walkthrough does not match its native model")
    }

    fn translate_seed(&self, value: &mut Value, key: &str, parent: &str) {
        match value {
            Value::String(source) => {
                let authored = matches!(
                    key,
                    "headline"
                        | "label"
                        | "description"
                        | "problem"
                        | "competencyLabel"
                        | "text"
                        | "ack"
                        | "exampleProblems"
                        | "questions"
                        | "checks"
                        | "watchOuts"
                        | "reasons"
                ) || (key == "name" && parent == "workflow");
                if authored {
                    *source = self.t(source, &[]);
                }
            }
            Value::Array(items) => {
                for item in items {
                    self.translate_seed(item, key, parent);
                }
            }
            Value::Object(fields) => {
                for (child_key, field) in fields {
                    self.translate_seed(field, child_key, key);
                }
            }
            _ => {}
        }
    }

    pub fn live_brief(&self, conversation: &str) -> Option<AgentBrief> {
        let mut brief = self.assets.live_briefs.get(conversation)?.clone();
        brief.locale = serde_json::from_value(Value::String(self.locale().to_owned())).ok()?;
        Some(brief)
    }

    pub fn agent_locale(&self) -> Locale {
        serde_json::from_value(Value::String(self.locale().to_owned())).unwrap_or_default()
    }
}

fn resolve_locale(input: &str) -> &'static str {
    let normalized = input
        .trim()
        .split('.')
        .next()
        .unwrap_or_default()
        .replace('_', "-")
        .to_ascii_lowercase();
    let mut parts = normalized.split('-');
    let language = parts.next().unwrap_or_default();
    if language == "zh" {
        if parts.any(|part| matches!(part, "hant" | "tw" | "hk" | "mo")) {
            return "en";
        }
        return "zh-Hans";
    }
    match language {
        "fr" => "fr",
        "es" => "es",
        "de" => "de",
        "it" => "it",
        "ko" => "ko",
        "ja" => "ja",
        "da" => "da",
        "uk" => "uk",
        "pl" => "pl",
        "ro" => "ro",
        "no" | "nb" => "nb",
        "fi" => "fi",
        "id" => "id",
        "fil" | "tl" => "tl",
        "ta" => "ta",
        "hi" => "hi",
        _ => "en",
    }
}

fn preference_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("Library/Application Support/com.agely.app/locale.json"))
}

pub fn load_locale() -> String {
    if let Ok(locale) = std::env::var("AGELY_LOCALE") {
        return resolve_locale(&locale).to_owned();
    }
    if let Some(path) = preference_path()
        && let Ok(text) = std::fs::read_to_string(path)
        && let Ok(Value::Object(values)) = serde_json::from_str::<Value>(&text)
        && let Some(Value::String(locale)) = values.get("locale")
    {
        return resolve_locale(locale).to_owned();
    }
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(locale) = std::env::var(key) {
            return resolve_locale(&locale).to_owned();
        }
    }
    "en".to_owned()
}

/// Persist only the chosen language. A failed save must not block the current language change.
pub fn save_locale(locale: &str) -> io::Result<()> {
    let Some(path) = preference_path() else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Home folder is unavailable",
        ));
    };
    let Some(parent) = path.parent() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Language preference folder is invalid",
        ));
    };
    std::fs::create_dir_all(parent)?;
    let temporary = parent.join(format!("locale-{}.json.tmp", std::process::id()));
    std::fs::write(
        &temporary,
        serde_json::to_vec(&serde_json::json!({ "locale": resolve_locale(locale) }))?,
    )?;
    std::fs::rename(temporary, path)
}

/// Whole ungrouped USD, matching the web app's accepted full-width, Devanagari and Tamil input.
pub fn parse_dollars(input: &str) -> Option<i64> {
    let mut decimal = String::new();
    let normalized: String = input.nfkc().collect();
    for character in normalized
        .trim()
        .strip_prefix('$')
        .unwrap_or(normalized.trim())
        .chars()
    {
        let digit = match character {
            '0'..='9' => character as u32 - '0' as u32,
            '\u{ff10}'..='\u{ff19}' => character as u32 - 0xff10,
            '\u{0966}'..='\u{096f}' => character as u32 - 0x0966,
            '\u{0be6}'..='\u{0bef}' => character as u32 - 0x0be6,
            _ => return None,
        };
        decimal.push(char::from_digit(digit, 10)?);
    }
    decimal.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Date, Month};

    #[test]
    fn translation_keeps_slots_and_falls_back_to_english() {
        let mut fr = I18n::new("fr-FR").unwrap();
        assert_eq!(fr.t("Language", &[]), "Langue");
        assert_eq!(
            fr.t("An unknown message {0}", &["42".into()]),
            "An unknown message 42"
        );
        assert_eq!(
            fr.t(
                "Step {0} of {1}{2}",
                &["1".into(), "13".into(), String::new()]
            ),
            "Étape 1 sur 13"
        );
        Arc::get_mut(&mut fr.assets)
            .unwrap()
            .catalogs
            .get_mut("fr")
            .unwrap()
            .insert("Amount: {0}".into(), "Montant inconnu".into());
        assert_eq!(fr.t("Amount: {0}", &["$150".into()]), "Amount: $150");
    }

    #[test]
    fn money_and_numbers_use_locale_grouping_and_currency_position() {
        let fr = I18n::new("fr").unwrap();
        assert_eq!(fr.money(150), "150\u{00a0}$US");
        assert_eq!(fr.number(12345), "12\u{202f}345");
        let hi = I18n::new("hi").unwrap();
        assert_eq!(hi.money(123456789), "$12,34,56,789");
        assert_eq!(I18n::new("es").unwrap().number(1000), "1000");
        assert_eq!(
            I18n::new("en").unwrap().money(i64::MIN),
            "-$9,223,372,036,854,775,808"
        );
    }

    #[test]
    fn seeded_content_translates_prose_but_keeps_authored_identifiers() {
        let seed: Value = I18n::new("ta").unwrap().localized_seed().unwrap();
        assert_eq!(seed["margaret"]["id"], "margaret");
        assert_eq!(seed["margaret"]["name"], "Margaret Okafor");
        assert_eq!(
            seed["margaret"]["competencies"][0]["tag"],
            "contract-manufacturing"
        );
        assert_eq!(
            seed["margaret"]["competencies"][2]["source"]["name"],
            "Supplier audit checklist.pdf"
        );
        assert_ne!(
            seed["margaret"]["competencies"][0]["label"],
            "Contract manufacturing terms"
        );
        assert_ne!(
            seed["workflow"]["name"],
            "First read of a contract-manufacturing agreement"
        );
    }

    #[test]
    fn date_format_uses_the_same_weekday_month_and_clock_as_the_web() {
        let date = Date::from_calendar_date(2024, Month::November, 21)
            .unwrap()
            .with_hms(13, 5, 0)
            .unwrap()
            .assume_utc();
        assert_eq!(
            I18n::new("en").unwrap().session_time(date),
            "Thursday, November 21 at 1:05 PM"
        );
        assert_eq!(
            I18n::new("ja").unwrap().session_time(date),
            "11月21日 木曜日 13:05"
        );
    }

    #[test]
    fn native_whole_dollar_input_rejects_ambiguous_separators() {
        for value in ["$60", "６０", "＄６０", "௬௦", "६०", " 60 "] {
            assert_eq!(parse_dollars(value), Some(60));
        }
        for value in ["", "1,000", "60.0", "-1", "$ 60", "60 dollars"] {
            assert_eq!(parse_dollars(value), None);
        }
    }

    #[test]
    fn locale_aliases_and_live_briefs_share_the_supported_registry() {
        assert_eq!(I18n::new("fil-PH").unwrap().locale(), "tl");
        assert_eq!(I18n::new("zh-TW").unwrap().locale(), "en");
        let ta = I18n::new("ta").unwrap();
        assert_eq!(ta.locales().len(), 18);
        assert_eq!(ta.live_brief("interview").unwrap().locale, Locale::Tamil);
        assert!(ta.live_brief("missing").is_none());
    }
}
