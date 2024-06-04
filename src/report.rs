// The MIT License (MIT)
// Copyright © 2021 Aukbit Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use crate::{
    config::{RunMode, CONFIG},
    crunch::OnetData,
};
use log::{info, warn};
use rand::Rng;
use regex::Regex;
use std::collections::HashSet;
use subxt::{ext::sp_core::H256, utils::AccountId32};

pub type EraIndex = u32;
pub type PageIndex = u32;

#[derive(Debug, Default, Clone)]
pub struct Points {
    pub validator: u32,
    pub era_avg: f64,
    pub ci99_9_interval: (f64, f64),
    pub outlier_limits: (f64, f64),
}

#[derive(Debug, Clone)]
pub struct Payout {
    pub block_number: u32,
    pub extrinsic: H256,
    pub era_index: u32,
    pub validator_amount_value: u128,
    pub nominators_amount_value: u128,
    pub nominators_quantity: u32,
    pub points: Points,
}

#[derive(Debug, Clone)]
pub struct Batch {
    pub block_number: u32,
    pub extrinsic: H256,
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub stash: AccountId32,
    pub controller: Option<AccountId32>,
    pub name: String,
    pub parent_identity: String,
    pub has_identity: bool,
    pub is_active: bool,
    pub is_previous_era_already_claimed: bool,
    pub claimed: Vec<(EraIndex, PageIndex)>,
    pub unclaimed: Vec<(EraIndex, PageIndex)>,
    pub payouts: Vec<Payout>,
    pub warnings: Vec<String>,
    pub onet: Option<OnetData>,
}

impl Validator {
    pub fn new(stash: AccountId32) -> Validator {
        Validator {
            stash,
            controller: None,
            name: "".to_string(),
            parent_identity: "".to_string(),
            has_identity: false,
            is_active: false,
            is_previous_era_already_claimed: false,
            claimed: Vec::new(),
            unclaimed: Vec::new(),
            payouts: Vec::new(),
            warnings: Vec::new(),
            onet: None,
        }
    }
}

pub type Validators = Vec<Validator>;

#[derive(Debug, Clone)]
pub struct SignerDetails {
    pub account: AccountId32,
    pub name: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Network {
    pub active_era: EraIndex,
    pub name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
}

#[derive(Debug, Default, Clone)]
pub struct PayoutSummary {
    pub calls: u32,
    pub calls_succeeded: u32,
    pub calls_failed: u32,
    pub next_minimum_expected: u32,
    pub total_validators: u32,
    pub total_unclaimed_pages: u32,
}

#[derive(Debug, Default, Clone)]
pub struct NominationPoolsSummary {
    pub calls: u32,
    pub calls_succeeded: u32,
    pub calls_failed: u32,
    pub total_members: u32,
    pub batches: Vec<Batch>,
}

#[derive(Debug)]
pub struct RawData {
    pub network: Network,
    pub signer_details: SignerDetails,
    pub validators: Validators,
    pub payout_summary: PayoutSummary,
    pub pools_summary: Option<NominationPoolsSummary>,
}

type Body = Vec<String>;

#[derive(Clone, PartialEq)]
enum Verbosity {
    Short,
    Medium,
    Long,
}

impl Verbosity {
    fn from_config() -> Self {
        let config = CONFIG.clone();
        if config.is_short {
            return Self::Short;
        }
        if config.is_medium {
            return Self::Medium;
        }
        Self::Long
    }

    fn _is_short(&self) -> bool {
        *self == Verbosity::Short
    }

    fn is_medium(&self) -> bool {
        *self == Verbosity::Medium
    }

    fn is_long(&self) -> bool {
        *self == Verbosity::Long
    }
}

pub struct Report {
    body: Body,
    verbosity: Verbosity,
}

impl Report {
    pub fn new() -> Report {
        Report {
            body: Vec::new(),
            verbosity: Verbosity::from_config(),
        }
    }

    pub fn add_raw_text(&mut self, t: String) {
        self.body.push(t);
    }

    pub fn add_text(&mut self, t: String) {
        if self.verbosity.is_long() {
            self.add_raw_text(t);
        }
    }
    pub fn add_break(&mut self) {
        self.add_raw_text("".into());
    }

    pub fn message(&self) -> String {
        self.body.join("\n")
    }

    pub fn formatted_message(&self) -> String {
        self.body.join("<br>")
    }

    pub fn log(&self) {
        info!("__START__");
        for t in &self.body {
            info!("{}", t);
        }
        info!("__END__");
    }
}

impl From<RawData> for Report {
    /// Converts a Crunch `RawData` into a [`Report`].
    fn from(data: RawData) -> Report {
        let config = CONFIG.clone();
        let mut report = Report::new();

        let summary_crunch_desc = if data.payout_summary.calls_succeeded > 0 {
            if config.group_identity_enabled {
                format!(
                    "{} crunched <b>{}</b> ({:.0}%) → ",
                    data.validators[0].parent_identity.clone(),
                    data.payout_summary.calls_succeeded,
                    (data.payout_summary.calls_succeeded as f32
                        / data.payout_summary.calls as f32)
                        * 100.0,
                )
            } else {
                format!(
                    "Crunched <b>{}</b> ({:.0}%) → ",
                    data.payout_summary.calls_succeeded,
                    (data.payout_summary.calls_succeeded as f32
                        / data.payout_summary.calls as f32)
                        * 100.0,
                )
            }
        } else {
            format!("")
        };

        let mut prefix = "Next".to_string();

        if data.payout_summary.calls_succeeded == 0 && config.group_identity_enabled {
            prefix = format!("{} next", data.validators[0].parent_identity);
        }

        let summary_next_desc = if data.payout_summary.next_minimum_expected > 0 {
            format!(
                "{} era expect <b>{}</b> ({:.0}%) {}",
                prefix,
                data.payout_summary.next_minimum_expected,
                (data.payout_summary.next_minimum_expected as f32
                    / data.payout_summary.total_validators as f32)
                    * 100.0,
                Random::Happy,
            )
        } else {
            format!("{} era expect <b>NO</b> rewards {}", prefix, Random::Grumpy)
        };

        report.add_raw_text(format!(
            "<details><summary>{}{}</summary>",
            summary_crunch_desc, summary_next_desc,
        ));

        report.add_raw_text("——".to_string());

        // Crunch Hello message
        report.add_text(format!("👋 {}!", Random::Hello));
        // Crunch package
        report.add_raw_text(format!(
            "🤖 <code>{} v{}</code>",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));

        // Network info
        report.add_break();
        report.add_raw_text(format!(
            "💙 <b>{}</b> is playing era <i>{}</i> 🎶 ",
            data.network.name, data.network.active_era
        ));

        // Signer
        report.add_text(format!(
            "<br>✍️ Signer &middot; <code>{}</code>",
            data.signer_details.name
        ));
        for warning in data.signer_details.warnings {
            report.add_raw_text(format!("⚠️ {} ⚠️", warning.clone()));
            warn!("{}", warning);
        }

        // Validators info
        for validator in data.validators {
            report.add_break();
            let is_active_desc = if validator.is_active { "🟢" } else { "🔴" };
            report.add_raw_text(format!(
                "{} <b><a href=\"https://{}.subscan.io/validator/{}\">{}</a></b>",
                is_active_desc,
                data.network.name.to_lowercase().trim().replace(" ", ""),
                validator.stash,
                validator.name,
            ));
            // Show validator warnings
            if validator.warnings.len() > 0 {
                for warning in validator.warnings {
                    report.add_raw_text(format!("⚠️ {} ⚠️", warning.clone()));
                    warn!("{}", warning);
                }
                continue;
            }

            report.add_text(format!(
                "💰 Stash &middot; <code>{}</code>",
                validator.stash
            ));

            // Check if there are no payouts
            if validator.payouts.len() == 0 {
                if validator.is_active {
                    report.add_text(format!(
                        "🥣 Looking forward for next <code>crunch</code> {} {}",
                        Random::Happy,
                        Random::Sport
                    ));
                } else {
                    report.add_text(format!(
                        "🥣 Nothing to <code>crunch</code> {}",
                        Random::Grumpy
                    ));
                }
            } else {
                // Show Validator payout info
                for payout in validator.payouts {
                    // Points
                    let reward_amount = format!(
                        "{:.4} {} {}",
                        (payout.validator_amount_value + payout.nominators_amount_value)
                            as f64
                            / 10f64.powi(data.network.token_decimals.into()),
                        data.network.token_symbol,
                        good_performance(
                            payout.points.validator.into(),
                            payout.points.ci99_9_interval.1,
                            payout.points.outlier_limits.1,
                        )
                    );

                    // Validator reward amount
                    let stash_amount = format!(
                        "{:.4} {}",
                        payout.validator_amount_value as f64
                            / 10f64.powi(data.network.token_decimals.into()),
                        data.network.token_symbol
                    );
                    let stash_amount_percentage = (payout.validator_amount_value as f64
                        / (payout.validator_amount_value + payout.nominators_amount_value)
                            as f64)
                        * 100.0;

                    // NOTE: if 'medium' flag is selected show validator total rewards alongside points
                    if report.verbosity.is_medium() {
                        report.add_raw_text(format!(
                            "🎲 Points {} {} ({:.0}) → 💸 {} → {} ({:.2}%)",
                            payout.points.validator,
                            trend(payout.points.validator.into(), payout.points.era_avg),
                            payout.points.era_avg,
                            reward_amount,
                            stash_amount,
                            stash_amount_percentage
                        ));
                    } else {
                        report.add_raw_text(format!(
                            "🎲 Points {} {} ({:.0}) → 💸 {}",
                            payout.points.validator,
                            trend(payout.points.validator.into(), payout.points.era_avg),
                            payout.points.era_avg,
                            reward_amount
                        ));
                    }

                    report.add_text(format!(
                        "🧑‍🚀 {} → 💸 <b>{}</b> ({:.2}%)",
                        validator.name, stash_amount, stash_amount_percentage
                    ));

                    // Nominators reward amount
                    let nominators_amount = format!(
                        "{:.4} {}",
                        payout.nominators_amount_value as f64
                            / 10f64.powi(data.network.token_decimals.into()),
                        data.network.token_symbol
                    );
                    let nominators_amount_percentage = (payout.nominators_amount_value
                        as f64
                        / (payout.validator_amount_value + payout.nominators_amount_value)
                            as f64)
                        * 100.0;
                    report.add_text(format!(
                        "🦸 Nominators ({}) → 💸 {} ({:.2}%)",
                        payout.nominators_quantity,
                        nominators_amount,
                        nominators_amount_percentage
                    ));

                    // Block number
                    report.add_raw_text(format!(
                        "💯 Payout for era <del>{}</del> finalized at block #{}
                        (<a href=\"https://{}.subscan.io/extrinsic/{:?}\">{}</a>) ✨",
                        payout.era_index,
                        payout.block_number,
                        data.network.name.to_lowercase().trim().replace(" ", ""),
                        payout.extrinsic,
                        payout.extrinsic.to_string()
                    ));
                }

                // Check if there are still eras left to claim
                if validator.unclaimed.len() > 0 {
                    let symbols = number_to_symbols(validator.unclaimed.len(), "⚡", 84);
                    report.add_text(format!(
						"{} There are still {} eras left with {} to <code>crunch</code> {}",
						symbols,
						validator.unclaimed.len(),
						context(),
						symbols
					));
                } else {
                    report.add_text(format!(
                        "✌️ {} just run out of {} 💫 💙",
                        validator.name,
                        context()
                    ));
                }
            }

            // General stats

            // Inclusion
            let eras_claimed = validator
                .claimed
                .clone()
                .into_iter()
                .map(|(era_index, _page_index)| era_index)
                .collect::<HashSet<EraIndex>>()
                .len();

            let eras_unclaimed = validator
                .unclaimed
                .clone()
                .into_iter()
                .map(|(era_index, _page_index)| era_index)
                .collect::<HashSet<EraIndex>>()
                .len();

            let inclusion_percentage =
                ((eras_claimed + eras_unclaimed) as f32 / 84.0) * 100.0;

            report.add_text(format!(
                "📒 Inclusion {}/{} ({:.2}%)",
                eras_claimed + eras_unclaimed,
                84,
                inclusion_percentage.clone()
            ));

            // Claimed
            if validator.claimed.len() > 0 {
                let claimed_percentage = (validator.claimed.len() as f32
                    / (validator.claimed.len() + validator.unclaimed.len()) as f32)
                    * 100.0;
                report.add_text(format!(
                    "😋 Crunched {}/{} ({:.2}%)",
                    validator.claimed.len(),
                    validator.claimed.len() + validator.unclaimed.len(),
                    claimed_percentage
                ));
            }

            // ONE-T stats
            if let Some(onet) = validator.onet {
                let para_inclusion = ((config.onet_number_last_sessions as f64
                    * onet.para_authority_inclusion)
                    .ceil()) as u32;
                if para_inclusion > 0 {
                    report.add_raw_text(format!(
                        "🎓 Grade from {}/{} sessions: <b>{}</b>",
                        para_inclusion, config.onet_number_last_sessions, onet.grade
                    ));
                }
            }
        }

        report.add_break();

        // Nomination Pools compound info
        if (config.pool_members_compound_enabled
            || config.pool_only_operator_compound_enabled)
            && data.pools_summary.is_some()
        {
            let pool_summary_data = data.pools_summary.unwrap();

            let threshold = format!(
                "{:.4} {}",
                config.pool_compound_threshold as f64
                    / 10f64.powi(data.network.token_decimals.into()),
                data.network.token_symbol,
            );

            let pools_desc = if config.pool_ids.len() == 1 {
                format!("Pool {}", config.pool_ids.get(0).unwrap())
            } else {
                format!("Pools {:?}", config.pool_ids)
            };

            if pool_summary_data.total_members > 0 {
                let members_desc = if pool_summary_data.total_members == 1 {
                    format!("1 reward")
                } else {
                    format!("{} rewards", pool_summary_data.total_members)
                };

                if config.pool_only_operator_compound_enabled {
                    report.add_raw_text(format!(
                        "♻️ Pool operator reward compounded from {}",
                        pools_desc
                    ));
                } else {
                    report.add_raw_text(format!(
                        "♻️ {} compounded from {}",
                        members_desc, pools_desc
                    ));
                }

                for batch in pool_summary_data.batches {
                    report.add_raw_text(format!(
                        "💯 Batch finalized at block #{}
                    (<a href=\"https://{}.subscan.io/extrinsic/{:?}\">{}</a>) ✨",
                        batch.block_number,
                        data.network.name.to_lowercase().trim().replace(" ", ""),
                        batch.extrinsic,
                        batch.extrinsic.to_string()
                    ));
                }
            } else {
                // NOTE: Just log if there are no pending rewards to compound
                info!(
                    "♻️ No pending rewards to compound above {} from {}",
                    threshold, pools_desc,
                );
            }
            report.add_break();
        }

        match config.run_mode {
            RunMode::Daily | RunMode::Turbo => {
                report.add_raw_text(format!(
                    "💤 The next <code>crunch</code> time will be in {} hours ⏱️",
                    config.interval / 3600
                ));
            }
            RunMode::Era => {
                report.add_raw_text(format!(
                    "💤 Until next era <i>{}</i> → Stay tuned 👀",
                    data.network.active_era + 1
                ));
            }
            _ => {}
        }

        report.add_raw_text("——".into());
        report.add_break();
        report.add_raw_text("</details>".into());

        // Log report
        report.log();

        report
    }
}

pub fn replace_emoji_lowercase(string: &String) -> String {
    let regex = Regex::new(concat!(
        "[",
        "\u{01F600}-\u{01F64F}",
        "\u{01F300}-\u{01F5FF}",
        "\u{01F680}-\u{01F6FF}",
        "\u{01F1E0}-\u{01F1FF}",
        "\u{002702}-\u{0027B0}",
        "\u{0024C2}-\u{01F251}",
        "\u{002500}-\u{002BEF}",
        "\u{01f926}-\u{01f937}",
        "\u{010000}-\u{10ffff}",
        "\u{2640}-\u{2642}",
        "\u{2600}-\u{2B55}",
        "\u{200d}",
        "\u{23cf}",
        "\u{23e9}",
        "\u{231a}",
        "\u{fe0f}",
        "\u{3030}",
        "\u{20e3}",
        "\u{0020}",
        "\u{000D}",
        "\u{000A}",
        "]+",
    ))
    .unwrap();

    regex.replace_all(string, "").to_string().to_lowercase()
}

fn number_to_symbols(n: usize, symbol: &str, max: usize) -> String {
    let cap: usize = match n {
        n if n < (max / 4) as usize => 1,
        n if n < (max / 2) as usize => 2,
        n if n < max - (max / 4) as usize => 3,
        _ => 4,
    };
    let v = vec![""; cap + 1];
    v.join(symbol)
}

fn trend(a: f64, b: f64) -> String {
    if a > b {
        String::from("⬆️")
    } else {
        String::from("⇩")
    }
}

fn performance(a: f64, b: f64, out: String) -> Option<String> {
    if a > b {
        return Some(out);
    }
    None
}

fn good_performance(value: u32, higher_limit: f64, outlier_limit: f64) -> String {
    match performance(value.into(), outlier_limit, "🤑 🤯 🚀".into()) {
        Some(p) => p,
        None => match performance(value.into(), higher_limit, "😊 🔥".into()) {
            Some(p) => p,
            None => String::from(""),
        },
    }
}

enum Random {
    Sport,
    Grumpy,
    Happy,
    Words,
    Hello,
}

impl std::fmt::Display for Random {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sport => {
                let v = vec![
                    "⛷",
                    "🏂",
                    "🪂",
                    "🏋️",
                    "🤸‍♂️",
                    "⛹️",
                    "🏇",
                    "🏌️",
                    "🧘",
                    "🏄‍♂️",
                    "🏊‍♂️",
                    "🚣‍♂️",
                    "🧗‍♂️",
                    "🚴‍♂️",
                ];
                write!(f, "{}", v[random_index(v.len())])
            }
            Self::Grumpy => {
                let v = vec![
                    "🤔", "😞", "😔", "😟", "😕", "🙁", "😣", "😖", "😫", "😩", "🥺",
                    "😢", "😭", "😤", "😠", "😡", "🤬",
                ];
                write!(f, "{}", v[random_index(v.len())])
            }
            Self::Happy => {
                let v = vec![
                    "😀", "😃", "😁", "😆", "😅", "😊", "🙂", "😉", "😝", "😜", "😎",
                    "🤩", "🥳", "😏", "😬",
                ];
                write!(f, "{}", v[random_index(v.len())])
            }
            Self::Words => {
                let v = vec![
                    "delicious",
                    "tasty",
                    "mental",
                    "psycho",
                    "fruity",
                    "crazy",
                    "spicy",
                    "yummy",
                    "supernatural",
                    "juicy",
                    "super",
                    "mellow",
                    "sweet",
                    "nutty",
                    "insane",
                    "fantastic",
                    "unbelievable",
                    "incredible",
                ];
                write!(f, "{}", v[random_index(v.len())])
            }
            Self::Hello => {
                let v = vec![
                    "Hello", "Hey", "Olá", "Hola", "Ciao", "Salut", "Privet", "Nǐ hǎo",
                    "Yā, Yō", "Hallo", "Oi", "Anyoung", "Ahlan", "Halløj", "Habari",
                    "Hallo", "Yassou", "Cześć", "Halo", "Hai", "Hai", "Selam", "Hej",
                    "Hei",
                ];
                write!(f, "{}", v[random_index(v.len())])
            }
        }
    }
}

fn random_index(len: usize) -> usize {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..len - 1)
}

fn context() -> String {
    let config = CONFIG.clone();
    if config.is_boring {
        return String::from("rewards");
    }
    format!("{} flakes", Random::Words)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats;

    #[test]
    fn good_performance_emojis() {
        let v = vec![
            80.0, 840.0, 920.0, 1580.0, 1160.0, 80.0, 940.0, 40.0, 20.0, 80.0, 60.0,
            2680.0, 1480.0, 1020.0, 2280.0, 1120.0, 2100.0, 900.0, 1460.0, 1240.0, 940.0,
            2380.0, 3420.0, 1560.0, 100.0, 1400.0, 180.0, 80.0, 1560.0, 80.0, 40.0,
            2720.0, 1660.0, 20.0, 1740.0, 1780.0, 2360.0, 960.0, 2420.0, 1700.0, 1080.0,
            4840.0, 1160.0, 1620.0, 20.0, 1620.0, 1740.0, 1540.0, 100.0, 1240.0, 1260.0,
            40.0, 5940.0, 1620.0, 1560.0, 1740.0, 100.0, 2760.0, 880.0, 100.0, 1740.0,
            1700.0, 4680.0, 1520.0, 2160.0, 1280.0, 2540.0, 3160.0,
        ];
        let avg = stats::mean(&v);
        let ci99_9 = stats::confidence_interval_99_9(&v);
        let mut points: Vec<u32> = v.iter().map(|points| *points as u32).collect();
        let iqr_interval = stats::iqr_interval(&mut points);
        println!("{:?}", avg);
        println!("{:?}", ci99_9);
        println!("{:?}", iqr_interval);
        assert_eq!(good_performance(1, ci99_9.1, iqr_interval.1), "");
        assert_eq!(good_performance(2620, ci99_9.1, iqr_interval.1), "😊 🔥");
        assert_eq!(good_performance(3160, ci99_9.1, iqr_interval.1), "🤑 🤯 🚀");
    }
}
