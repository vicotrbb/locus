#![allow(missing_docs)]

use std::{env, sync::OnceLock};

static CRITERION_FILTER_TOKENS: OnceLock<Vec<String>> = OnceLock::new();

pub(crate) fn should_print_sample(sample_label: &str, benchmark_label: &str) -> bool {
    let filters = CRITERION_FILTER_TOKENS.get_or_init(criterion_filter_tokens);
    filters.is_empty()
        || filters
            .iter()
            .any(|filter| sample_label.contains(filter) || benchmark_label.contains(filter))
}

fn criterion_filter_tokens() -> Vec<String> {
    let mut args = env::args().skip(1);
    let mut filters = Vec::new();

    while let Some(arg) = args.next() {
        if arg == "--bench" || arg == "bench" {
            continue;
        }
        if arg.starts_with("--") {
            if criterion_option_takes_value(&arg) {
                let _ = args.next();
            }
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        filters.push(arg);
    }

    filters
}

fn criterion_option_takes_value(arg: &str) -> bool {
    !arg.contains('=')
        && matches!(
            arg,
            "--baseline"
                | "--color"
                | "--confidence-level"
                | "--measurement-time"
                | "--noise-threshold"
                | "--nresamples"
                | "--output-format"
                | "--plotting-backend"
                | "--profile-time"
                | "--sample-size"
                | "--save-baseline"
                | "--significance-level"
                | "--warm-up-time"
        )
}
