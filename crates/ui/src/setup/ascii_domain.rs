//! The ASCII form of a domain, for the server names guessed from an address.
//!
//! A domain with letters outside ASCII (`münchen.de`) is looked up under the
//! name made by punycode (`xn--mnchen-3ya.de`), and the host check on the form
//! takes ASCII only. A label that is already ASCII is left exactly as it is.
//! The domain is expected in lower case and in its composed form: no other
//! mapping or normalizing is done here.

const BASE: u32 = 36;
const T_MIN: u32 = 1;
const T_MAX: u32 = 26;
const SKEW: u32 = 38;
const DAMP: u32 = 700;
const FIRST_BIAS: u32 = 72;
const FIRST_CODE: u32 = 128;
/// The longest label a host name may have.
const MAX_LABEL: usize = 63;

/// The domain with every label that has non-ASCII letters turned into its
/// `xn--` form, or `None` when one cannot be (too long, or too large to
/// work out).
pub fn ascii_domain(domain: &str) -> Option<String> {
    let labels: Option<Vec<String>> = domain
        .split('.')
        .map(|label| if label.is_ascii() { Some(label.to_string()) } else { ascii_label(label) })
        .collect();
    Some(labels?.join("."))
}

fn ascii_label(label: &str) -> Option<String> {
    let label = format!("xn--{}", encode(label)?);
    (label.len() <= MAX_LABEL).then_some(label)
}

/// RFC 3492 punycode of one label.
fn encode(label: &str) -> Option<String> {
    let input: Vec<u32> = label.chars().map(u32::from).collect();
    let mut out: String = label.chars().filter(char::is_ascii).collect();
    let basic = u32::try_from(out.len()).ok()?;
    if basic > 0 {
        out.push('-');
    }
    let (mut code, mut delta, mut bias, mut handled) = (FIRST_CODE, 0u32, FIRST_BIAS, basic);
    while (handled as usize) < input.len() {
        let next = input.iter().copied().filter(|&ch| ch >= code).min()?;
        delta = delta.checked_add((next - code).checked_mul(handled + 1)?)?;
        code = next;
        for &ch in &input {
            if ch < code {
                delta = delta.checked_add(1)?;
            }
            if ch == code {
                push_number(&mut out, delta, bias);
                bias = adapt(delta, handled + 1, handled == basic);
                delta = 0;
                handled += 1;
            }
        }
        delta = delta.checked_add(1)?;
        code += 1;
    }
    Some(out)
}

/// `value` as a run of digits, least significant first, ending on a digit
/// that is below its threshold.
fn push_number(out: &mut String, value: u32, bias: u32) {
    let (mut rest, mut place) = (value, BASE);
    loop {
        let threshold = if place <= bias {
            T_MIN
        } else if place >= bias + T_MAX {
            T_MAX
        } else {
            place - bias
        };
        if rest < threshold {
            break;
        }
        out.push(digit(threshold + (rest - threshold) % (BASE - threshold)));
        rest = (rest - threshold) / (BASE - threshold);
        place += BASE;
    }
    out.push(digit(rest));
}

fn digit(value: u32) -> char {
    // Below 26 a letter, below 36 a number: `value` never reaches 36.
    char::from_u32(if value < 26 { 'a' as u32 + value } else { '0' as u32 + value - 26 }).unwrap_or('-')
}

fn adapt(delta: u32, points: u32, first: bool) -> u32 {
    let mut delta = if first { delta / DAMP } else { delta / 2 };
    delta += delta / points;
    let mut steps = 0;
    while delta > ((BASE - T_MIN) * T_MAX) / 2 {
        delta /= BASE - T_MIN;
        steps += BASE;
    }
    steps + (BASE - T_MIN + 1) * delta / (delta + SKEW)
}

#[cfg(test)]
include!("ascii_domain.tests.rs");
