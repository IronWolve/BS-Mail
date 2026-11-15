//! International domain names in their plain ASCII form, so that an address
//! such as `anna@münchen.de` can go out with an ASCII envelope and header.
//! Each label with characters outside ASCII is lowercased and written as
//! `xn--` followed by its Punycode (RFC 3492).
//!
//! Only letters and digits are taken: anything else, including combining
//! marks and look-alike dots, is refused rather than mapped, so a domain is
//! never turned into one the person did not type. Only the part after the `@`
//! is converted; a local part outside ASCII needs SMTPUTF8, which is not built.

const BASE: u32 = 36;
const T_MIN: u32 = 1;
const T_MAX: u32 = 26;
const SKEW: u32 = 38;
const DAMP: u32 = 700;
const INITIAL_BIAS: u32 = 72;
const INITIAL_N: u32 = 128;

/// The ASCII form of a domain, or `None` when a label cannot be written that
/// way. Labels that are plain ASCII already are left exactly as they are.
pub fn ascii_domain(domain: &str) -> Option<String> {
    let mut labels = Vec::new();
    for label in domain.split('.') {
        labels.push(if label.is_ascii() { label.to_owned() } else { ascii_label(label)? });
    }
    Some(labels.join("."))
}

/// One label with characters outside ASCII, as `xn--...`.
fn ascii_label(label: &str) -> Option<String> {
    let lower = label.to_lowercase();
    let letters_only = lower.chars().all(|c| c == '-' || c.is_alphanumeric());
    let bad_hyphens = lower.starts_with('-') || lower.ends_with('-') || lower.get(2..4) == Some("--");
    if !letters_only || bad_hyphens {
        return None;
    }
    Some(format!("xn--{}", encode(&lower)?))
}

/// RFC 3492 section 6.3, for a label already checked.
fn encode(label: &str) -> Option<String> {
    let input: Vec<u32> = label.chars().map(u32::from).collect();
    let mut output: String = label.chars().filter(char::is_ascii).collect();
    let basic = u32::try_from(output.len()).ok()?;
    if basic > 0 {
        output.push('-');
    }
    let (mut n, mut delta, mut bias, mut handled) = (INITIAL_N, 0u32, INITIAL_BIAS, basic);
    while (handled as usize) < input.len() {
        let next = input.iter().copied().filter(|&c| c >= n).min()?;
        delta = delta.checked_add((next - n).checked_mul(handled + 1)?)?;
        n = next;
        for &c in &input {
            if c < n {
                delta = delta.checked_add(1)?;
            }
            if c == n {
                push_number(&mut output, delta, bias);
                bias = adapt(delta, handled + 1, handled == basic);
                delta = 0;
                handled += 1;
            }
        }
        delta += 1;
        n += 1;
    }
    Some(output)
}

/// Writes `value` as a variable-length number in base 36.
fn push_number(output: &mut String, value: u32, bias: u32) {
    let (mut rest, mut k) = (value, BASE);
    loop {
        let t = threshold(k, bias);
        if rest < t {
            break;
        }
        output.push(digit(t + (rest - t) % (BASE - t)));
        rest = (rest - t) / (BASE - t);
        k += BASE;
    }
    output.push(digit(rest));
}

fn threshold(k: u32, bias: u32) -> u32 {
    if k <= bias {
        T_MIN
    } else if k >= bias + T_MAX {
        T_MAX
    } else {
        k - bias
    }
}

/// How the bias moves after each number, so that common code points stay short.
fn adapt(delta: u32, points: u32, first: bool) -> u32 {
    let mut delta = if first { delta / DAMP } else { delta / 2 };
    delta += delta / points;
    let mut k = 0;
    while delta > ((BASE - T_MIN) * T_MAX) / 2 {
        delta /= BASE - T_MIN;
        k += BASE;
    }
    k + (BASE - T_MIN + 1) * delta / (delta + SKEW)
}

fn digit(value: u32) -> char {
    let byte = if value < 26 { b'a' + value as u8 } else { b'0' + (value - 26) as u8 };
    char::from(byte)
}

#[cfg(test)]
include!("idn.tests.rs");
