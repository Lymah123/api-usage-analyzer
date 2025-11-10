
pub fn format_currency(value: f64) -> String {
    format!("${:.2}", value)
}

pub fn format_tokens(tokens: i64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.2}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.2}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

pub fn parse_period_to_days(period: &str) -> i64 {
    match period {
        "24h" | "1d" => 1,
        "7d" => 7,
        "30d" => 30,
        "90d" => 90,
        _ => 7,
    }
}
