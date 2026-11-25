//! Decimal input parsing and display. Stored prices are exact decimals; only conversions go through f64.

use rust_decimal::prelude::*;
use rust_decimal::RoundingStrategy;

/// Parses what a person types as a price: `9.99`, `9,99`, `1 299,50`, `1,299.50`, `1.299,50`.
/// A lone separator followed by exactly three digits after a short non-zero head ("1,299") is a thousands group.
/// Returns `None` for empty, negative, malformed or absurdly large input.
pub fn parse_price(input: &str) -> Option<Decimal> {
    let s: String = input.trim().chars().filter(|c| !c.is_whitespace() && !matches!(*c, '\u{a0}' | '\'' | '\u{202f}')).collect();
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
        return None;
    }
    let count = |ch: char| s.matches(ch).count();
    let (dots, commas) = (count('.'), count(','));
    // (decimal separator, group separator)
    let (dec, group): (Option<char>, Option<char>) = match (dots, commas) {
        (0, 0) => (None, None),
        (d, c) if d > 0 && c > 0 => {
            let (last_dot, last_comma) = (s.rfind('.')?, s.rfind(',')?);
            let dec = if last_dot > last_comma { '.' } else { ',' };
            if count(dec) != 1 {
                return None;
            }
            (Some(dec), Some(if dec == '.' { ',' } else { '.' }))
        }
        (d, 0) => single(&s, '.', d),
        (0, c) => single(&s, ',', c),
        _ => return None,
    };
    let mut int = s.clone();
    let mut frac = String::new();
    if let Some(dc) = dec {
        let (a, b) = s.rsplit_once(dc)?;
        int = a.to_string();
        frac = b.to_string();
    }
    if let Some(g) = group {
        let parts: Vec<&str> = int.split(g).collect();
        let ok = parts[0].len() <= 3 && parts[1..].iter().all(|p| p.len() == 3);
        if parts.len() > 1 && !ok {
            return None;
        }
        int = parts.concat();
    }
    if int.contains(['.', ',']) || frac.contains(['.', ',']) {
        return None;
    }
    if int.is_empty() && frac.is_empty() {
        return None;
    }
    if int.is_empty() {
        int.push('0');
    }
    let text = if frac.is_empty() { int } else { format!("{int}.{frac}") };
    if text.len() > 16 {
        return None;
    }
    Some(Decimal::from_str_exact(&text).ok()?.normalize())
}

/// A string with only one kind of separator: is it a decimal mark or a thousands group?
fn single(s: &str, ch: char, n: usize) -> (Option<char>, Option<char>) {
    if n > 1 {
        return (None, Some(ch));
    }
    let at = s.find(ch).unwrap_or(0);
    let (head, tail) = (&s[..at], &s[at + 1..]);
    if tail.len() == 3 && (1..=3).contains(&head.len()) && !head.starts_with('0') {
        (None, Some(ch))
    } else {
        (Some(ch), None)
    }
}

/// Rounds half to even to `dec` places.
pub fn round(d: Decimal, dec: u32) -> Decimal {
    d.round_dp_with_strategy(dec, RoundingStrategy::MidpointNearestEven)
}

/// A figure for display: rounded half-even, a thin space between thousands, `.` or `,` as the decimal mark.
pub fn format_amount(d: Decimal, dec: u32, comma: bool) -> String {
    let r = round(d, dec);
    let neg = r.is_sign_negative() && !r.is_zero();
    let text = format!("{:.*}", dec as usize, r.abs());
    let (int, frac) = match text.split_once('.') {
        Some((i, f)) => (i.to_string(), Some(f.to_string())),
        None => (text, None),
    };
    let mut grouped = String::new();
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            grouped.push('\u{202f}');
        }
        grouped.push(ch);
    }
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    out.push_str(&grouped);
    if let Some(f) = frac {
        out.push(if comma { ',' } else { '.' });
        out.push_str(&f);
    }
    out
}

/// "₴199.00" style text for a price in its own currency (symbol before the figure, decimals per currency).
/// Currencies whose symbol is just their code ("KWD") put it after the figure.
pub fn format_money(d: Decimal, code: &str, comma: bool) -> String {
    let sym = super::currency::symbol(code);
    let n = format_amount(d, super::currency::decimals(code), comma);
    if sym.chars().count() >= 3 && sym.chars().all(|c| c.is_ascii_uppercase()) {
        format!("{n} {sym}")
    } else {
        format!("{sym}{n}")
    }
}

/// The text a price field starts with when editing: the plain number with the locale's decimal mark.
pub fn edit_text(d: Decimal, comma: bool) -> String {
    let s = d.normalize().to_string();
    if comma {
        s.replace('.', ",")
    } else {
        s
    }
}

/// Decimal to f64 for conversion maths.
pub fn to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

/// f64 back to a decimal (conversion results only).
pub fn from_f64(x: f64) -> Option<Decimal> {
    if x.is_finite() {
        Decimal::from_f64(x)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Option<String> {
        parse_price(s).map(|d| d.to_string())
    }

    #[test]
    fn both_decimal_separators() {
        assert_eq!(p("9.99").as_deref(), Some("9.99"));
        assert_eq!(p("9,99").as_deref(), Some("9.99"));
        assert_eq!(p(" 199 ").as_deref(), Some("199"));
        assert_eq!(p("0,5").as_deref(), Some("0.5"));
        assert_eq!(p(".5").as_deref(), Some("0.5"));
        assert_eq!(p("12.50").as_deref(), Some("12.5"));
        assert_eq!(p("0.125").as_deref(), Some("0.125"));
    }

    #[test]
    fn thousands_separators() {
        assert_eq!(p("1 299,50").as_deref(), Some("1299.5"));
        assert_eq!(p("1,299.50").as_deref(), Some("1299.5"));
        assert_eq!(p("1.299,50").as_deref(), Some("1299.5"));
        assert_eq!(p("1,299").as_deref(), Some("1299"));
        assert_eq!(p("1.234.567").as_deref(), Some("1234567"));
        assert_eq!(p("1\u{a0}299").as_deref(), Some("1299"));
    }

    #[test]
    fn rejects_garbage() {
        for bad in ["", "  ", "abc", "-5", "1.2.3,4", "9,99,1", "$5", "1e5", ".", ",", "12.34.56"] {
            assert!(parse_price(bad).is_none(), "{bad:?} must not parse");
        }
        assert!(parse_price("99999999999999999").is_none());
    }

    #[test]
    fn rounding_is_half_even() {
        assert_eq!(round(Decimal::from_str("2.345").unwrap(), 2).to_string(), "2.34");
        assert_eq!(round(Decimal::from_str("2.355").unwrap(), 2).to_string(), "2.36");
        assert_eq!(round(Decimal::from_str("0.5").unwrap(), 0).to_string(), "0");
        assert_eq!(round(Decimal::from_str("1.5").unwrap(), 0).to_string(), "2");
        assert_eq!(round(Decimal::from_str("2.5").unwrap(), 0).to_string(), "2");
    }

    #[test]
    fn formatting() {
        let d = Decimal::from_str("1234567.891").unwrap();
        assert_eq!(format_amount(d, 2, false), "1\u{202f}234\u{202f}567.89");
        assert_eq!(format_amount(d, 2, true), "1\u{202f}234\u{202f}567,89");
        assert_eq!(format_amount(d, 0, false), "1\u{202f}234\u{202f}568");
        assert_eq!(format_amount(Decimal::from_str("5").unwrap(), 2, false), "5.00");
        assert_eq!(format_money(Decimal::from_str("9.99").unwrap(), "USD", false), "$9.99");
        assert_eq!(format_money(Decimal::from_str("1500").unwrap(), "JPY", false), "¥1\u{202f}500");
        assert_eq!(format_money(Decimal::from_str("3").unwrap(), "KWD", false), "3.000 KWD");
        assert_eq!(edit_text(Decimal::from_str("9.90").unwrap(), true), "9,9");
    }
}
