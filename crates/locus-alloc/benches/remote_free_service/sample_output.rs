#![allow(missing_docs)]

use std::{env, fmt, fmt::Write as _, sync::OnceLock};

const JSON_OUTPUT_ENV: &str = "LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON";
const SAMPLE_SCHEMA: &str = "locus.remote_free_service.telemetry.sample.v1";

static JSON_OUTPUT_ENABLED: OnceLock<bool> = OnceLock::new();

pub(crate) fn print_sample_line(benchmark_label: &str, args: fmt::Arguments<'_>) {
    let line = args.to_string();
    println!("{line}");

    if json_output_enabled() {
        println!("{}", sample_line_json(benchmark_label, &line));
    }
}

fn json_output_enabled() -> bool {
    *JSON_OUTPUT_ENABLED.get_or_init(|| {
        env::var(JSON_OUTPUT_ENV).is_ok_and(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on" | "json"
            )
        })
    })
}

fn sample_line_json(benchmark_label: &str, line: &str) -> String {
    let sample_label = line.split_whitespace().next().unwrap_or_default();
    let mut output = String::new();

    output.push('{');
    push_json_field(&mut output, "schema", SAMPLE_SCHEMA, true);
    push_json_field(&mut output, "benchmark", benchmark_label, false);
    push_json_field(&mut output, "sample", sample_label, false);
    push_json_field(&mut output, "line", line, false);
    output.push_str(",\"fields\":{");

    let mut first_field = true;
    for token in line.split_whitespace().skip(1) {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };

        if !first_field {
            output.push(',');
        }
        first_field = false;

        push_json_string(&mut output, key);
        output.push(':');
        push_json_value(&mut output, value);
    }

    output.push_str("}}");
    output
}

fn push_json_field(output: &mut String, key: &str, value: &str, first: bool) {
    if !first {
        output.push(',');
    }
    push_json_string(output, key);
    output.push(':');
    push_json_string(output, value);
}

fn push_json_value(output: &mut String, value: &str) {
    match value {
        "true" | "false" => output.push_str(value),
        value if is_json_number(value) => output.push_str(value),
        value => push_json_string(output, value),
    }
}

fn is_json_number(value: &str) -> bool {
    let Some((head, tail)) = value.split_once('.') else {
        return value.bytes().all(|byte| byte.is_ascii_digit()) && !value.is_empty();
    };

    !head.is_empty()
        && !tail.is_empty()
        && head.bytes().all(|byte| byte.is_ascii_digit())
        && tail.bytes().all(|byte| byte.is_ascii_digit())
}

fn push_json_string(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch < ' ' => {
                output.push_str("\\u");
                let _ = write!(output, "{:04x}", ch as u32);
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
}
