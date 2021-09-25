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
use crate::config::CONFIG;
use crate::crunch::RawData;
use log::{info, warn};
use rand::Rng;

type Body = Vec<String>;

pub struct Report {
    body: Body,
    is_short: bool,
}

impl Report {
    pub fn new() -> Report {
        let config = CONFIG.clone();
        Report {
            body: Vec::new(),
            is_short: config.is_short,
        }
    }

    pub fn add_raw_text(&mut self, t: String) {
        self.body.push(t);
    }

    pub fn add_text(&mut self, t: String) {
        if !self.is_short {
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
        self.body.join("<br/>")
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
        let mut report = Report::new();
        // Crunch Hello message
        report.add_raw_text(format!("👋 {}!", Random::Hello));
        
        // Network info
        report.add_raw_text(format!(
            "💙 <b>{}</b> is playing era <i>{}</i> 🎶 ",
            data.network.name, data.network.active_era
        ));

        // Signer
        report.add_break();
        report.add_text(format!(
            "✍️ Signer &middot; <code>{}</code>",
            data.signer.name
        ));
        for warning in data.signer.warnings {
            report.add_raw_text(warning.clone());
            warn!("{}", warning);
        }

        // Validators info
        for validator in data.validators {
            report.add_break();
            // Show validator warnings
            if validator.warnings.len() > 0 {
                for warning in validator.warnings {
                    report.add_raw_text(format!("⚠️ {} ⚠️", warning.clone()));
                    warn!("{}", warning);
                }
                continue;
            }
            let is_active_desc = if validator.is_active { "🟢" } else { "🔴" };
            report.add_raw_text(format!(
                "{} <b><a href=\"https://{}.subscan.io/validator/{}\">{}</a></b>",
                is_active_desc,
                data.network.name.to_lowercase(),
                validator.stash,
                validator.name,
            ));
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
                        (payout.validator_amount_value + payout.nominators_amount_value) as f64
                            / 10f64.powi(data.network.token_decimals.into()),
                        data.network.token_symbol,
                        good_performance(
                            payout.points.validator.into(),
                            payout.points.ci99.1,
                            "🤯 🚀".into()
                        )
                    );

                    report.add_raw_text(format!(
                        "🎲 Points {} {}{}{} ({:.0}) -> 💸 {}",
                        payout.points.validator,
                        trend(payout.points.validator.into(), payout.points.era_avg),
                        good_performance(
                            payout.points.validator.into(),
                            payout.points.ci99.1,
                            "↑".into()
                        ),
                        poor_performance(
                            payout.points.validator.into(),
                            payout.points.ci99.0,
                            "↓".into()
                        ),
                        payout.points.era_avg,
                        reward_amount
                    ));
                    // Validator reward amount
                    let stash_amount = format!(
                        "{:.4} {}",
                        payout.validator_amount_value as f64
                            / 10f64.powi(data.network.token_decimals.into()),
                        data.network.token_symbol
                    );
                    let stash_amount_percentage = (payout.validator_amount_value as f64
                        / (payout.validator_amount_value + payout.nominators_amount_value) as f64)
                        * 100.0;
                    report.add_text(format!(
                        "🧑‍🚀 {} -> 💸 <b>{}</b> ({:.2}%)",
                        validator.name, stash_amount, stash_amount_percentage
                    ));

                    // Nominators reward amount
                    let nominators_amount = format!(
                        "{:.4} {}",
                        payout.nominators_amount_value as f64
                            / 10f64.powi(data.network.token_decimals.into()),
                        data.network.token_symbol
                    );
                    let nominators_amount_percentage = (payout.nominators_amount_value as f64
                        / (payout.validator_amount_value + payout.nominators_amount_value) as f64)
                        * 100.0;
                    report.add_text(format!(
                        "🦸 Nominators ({}) -> 💸 {} ({:.2}%)",
                        payout.nominators_quantity, nominators_amount, nominators_amount_percentage
                    ));

                    // Block number
                    report.add_raw_text(format!(
                        "💯 Payout for era <del>{}</del> finalized at block #{} 
                        (<a href=\"https://{}.subscan.io/extrinsic/{:?}\">{}</a>) ✨",
                        payout.era_index,
                        payout.block,
                        data.network.name.to_lowercase(),
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
            let inclusion_percentage =
                ((validator.claimed.len() + validator.unclaimed.len()) as f32 / 84.0) * 100.0;
            report.add_text(format!(
                "📒 Inclusion {}/{} ({:.2}%)",
                validator.claimed.len() + validator.unclaimed.len(),
                84,
                inclusion_percentage
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
        }

        report.add_break();

        let config = CONFIG.clone();
        if config.is_mode_era {
            report.add_raw_text(format!(
                "💨 Until next era <i>{}</i> -> Stay tuned 👀",
                data.network.active_era + 1
            ));
        } else {
            report.add_raw_text(format!(
                "💨 The next <code>crunch</code> time will be in {} hours ⏱️",
                config.interval / 3600
            ));
        };
        // Crunch package
        report.add_text(format!(
            "🤖 <code>{} v{}</code> 💤",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));
        report.add_raw_text("___".into());
        report.add_break();

        // Log report
        report.log();

        report
    }
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
        String::from("↑")
    } else {
        String::from("↓")
    }
}

fn good_performance(a: f64, b: f64, out: String) -> String {
    if a >= b {
        return out;
    }
    String::from("")
}

fn poor_performance(a: f64, b: f64, out: String) -> String {
    if a <= b {
        return out;
    }
    String::from("")
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
                    "🤔", "😞", "😔", "😟", "😕", "🙁", "😣", "😖", "😫", "😩", "🥺", "😢", "😭",
                    "😤", "😠", "😡", "🤬",
                ];
                write!(f, "{}", v[random_index(v.len())])
            }
            Self::Happy => {
                let v = vec![
                    "😀", "😃", "😁", "😆", "😅", "😊", "🙂", "😉", "😝", "😜", "😎", "🤩", "🥳",
                    "😏", "😬",
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
                    "Hello", "Hey", "Olá", "Hola", "Ciao", "Salut", "Privet", "Nǐ hǎo", "Yā, Yō",
                    "Hallo", "Oi", "Anyoung", "Ahlan", "Halløj", "Habari", "Hallo", "Yassou",
                    "Cześć", "Halo", "Hai", "Hai", "Selam", "Hej", "Hei",
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
