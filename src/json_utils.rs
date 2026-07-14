use rust_extensions::StrOrString;

const NULL_LC: [u8; 4] = [b'n', b'u', b'l', b'l'];
const NULL_UC: [u8; 4] = [b'N', b'U', b'L', b'L'];

pub fn is_null(src: &[u8]) -> bool {
    if is_that_value(&NULL_LC, &NULL_UC, src) {
        return true;
    }

    return false;
}

pub enum NumberType {
    NaN,
    Number,
    Double,
}

impl NumberType {
    pub fn is_nan(&self) -> bool {
        match self {
            NumberType::NaN => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            NumberType::Number => true,
            _ => false,
        }
    }

    pub fn is_double(&self) -> bool {
        match self {
            NumberType::Double => true,
            _ => false,
        }
    }
}

// Classifies a value slice according to the JSON number grammar:
//   number = [ '-' ] int [ frac ] [ exp ]
//   frac   = '.' digits
//   exp    = ('e' | 'E') [ '+' | '-' ] digits
//
// A value that contains a fraction ('.') or an exponent ('e'/'E') is a `Double`; a plain
// integer is a `Number`; anything else (including a lone sign / dot, or a value that does not
// hold at least one digit) is `NaN`. Both cases of the exponent marker and a signed exponent
// (`1e5`, `1.5E-3`, `1e+30`, `-2e3`) are accepted.
pub fn is_number(src: &[u8]) -> NumberType {
    let mut dots = 0;
    let mut exps = 0;
    let mut digits = 0;
    let mut exp_digits = 0;
    // true right after the exponent marker, so the sign of the exponent is accepted
    let mut after_exp_marker = false;

    for i in 0..src.len() {
        let c = src[i];

        if c == b'.' {
            // a dot is only valid in the mantissa, never after the exponent marker
            if exps > 0 {
                return NumberType::NaN;
            }
            dots += 1;
            if dots > 1 {
                return NumberType::NaN;
            }
            after_exp_marker = false;
            continue;
        }

        if c == b'e' || c == b'E' {
            exps += 1;
            // only one exponent, and it must follow at least one mantissa digit
            if exps > 1 || digits == 0 {
                return NumberType::NaN;
            }
            after_exp_marker = true;
            continue;
        }

        if c == b'-' || c == b'+' {
            // a sign is valid at the very start or immediately after the exponent marker
            if i == 0 || after_exp_marker {
                after_exp_marker = false;
                continue;
            }
            return NumberType::NaN;
        }

        if c >= b'0' && c <= b'9' {
            digits += 1;
            if exps > 0 {
                exp_digits += 1;
            }
            after_exp_marker = false;
            continue;
        }

        return NumberType::NaN;
    }

    // an all-sign / lone-dot / empty slice is not a number
    if digits == 0 {
        return NumberType::NaN;
    }

    // an exponent marker with no exponent digits (`1e`, `1e+`) is not a number
    if exps > 0 && exp_digits == 0 {
        return NumberType::NaN;
    }

    if dots == 0 && exps == 0 {
        return NumberType::Number;
    }

    NumberType::Double
}

pub fn is_that_value(src_lc: &[u8], src_uc: &[u8], dest: &[u8]) -> bool {
    if src_lc.len() != dest.len() {
        return false;
    }

    let mut pos = 0;

    for b in dest {
        if *b != src_lc[pos] && *b != src_uc[pos] {
            return false;
        }

        pos += 1;
    }

    return true;
}

const TRUE_LC: [u8; 4] = [b't', b'r', b'u', b'e'];
const TRUE_UC: [u8; 4] = [b'T', b'R', b'U', b'E'];

const FALSE_LC: [u8; 5] = [b'f', b'a', b'l', b's', b'e'];
const FALSE_UC: [u8; 5] = [b'F', b'A', b'L', b'S', b'E'];

pub fn is_bool(src: &[u8]) -> bool {
    if is_that_value(&TRUE_LC, &TRUE_UC, src) {
        return true;
    }

    if is_that_value(&FALSE_LC, &FALSE_UC, src) {
        return true;
    }

    false
}

pub fn as_bool_value(src: &[u8]) -> Option<bool> {
    if is_that_value(&TRUE_LC, &TRUE_UC, src) {
        return Some(true);
    }

    if is_that_value(&FALSE_LC, &FALSE_UC, src) {
        return Some(false);
    }

    None
}

pub fn is_array(src: &[u8]) -> bool {
    src[0] == crate::consts::OPEN_ARRAY
}

pub fn is_object(src: &[u8]) -> bool {
    src[0] == crate::consts::OPEN_BRACKET
}

pub fn is_string(src: &[u8]) -> bool {
    src[0] == '"' as u8 || src[0] == '\'' as u8
}

pub fn try_get_string_value<'s>(src: &'s [u8]) -> Option<StrOrString<'s>> {
    // Requires both surrounding quotes; a lone quote (len < 2) is not a well formed string.
    if src.len() >= 2 && is_string(src) {
        // Untrusted input can carry invalid UTF-8 inside the quotes - return `None` instead of
        // panicking (the previous `.unwrap()` was a DoS on hostile bytes).
        let inner = std::str::from_utf8(src[1..src.len() - 1].as_ref()).ok()?;
        return Some(crate::json_string_value::de_escape_json_string_value(inner));
    }

    None
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_null_str() {
        assert_eq!(false, is_null("15".as_bytes()));

        assert_eq!(true, is_null("null".as_bytes()));
        assert_eq!(true, is_null("Null".as_bytes()));
    }

    #[test]
    fn test_is_number() {
        assert!(is_number("15.5".as_bytes()).is_double());

        assert!(is_number("15".as_bytes()).is_number());

        assert!(is_number("+15.5".as_bytes()).is_double());

        assert!(is_number("-15.5".as_bytes()).is_double());

        assert!(is_number("10.0.0.3:5125".as_bytes()).is_nan());
    }

    #[test]
    fn test_is_number_exponents() {
        // exponent forms are valid JSON numbers and classify as Double
        assert!(is_number("1e5".as_bytes()).is_double());
        assert!(is_number("1E5".as_bytes()).is_double());
        assert!(is_number("1.5E-3".as_bytes()).is_double());
        assert!(is_number("1e+30".as_bytes()).is_double());
        assert!(is_number("-2e3".as_bytes()).is_double());
        assert!(is_number("6.022e23".as_bytes()).is_double());

        // malformed exponents are NaN
        assert!(is_number("e5".as_bytes()).is_nan()); // no mantissa digit
        assert!(is_number("1e".as_bytes()).is_nan()); // exponent marker, no exponent digit
        assert!(is_number("1e+".as_bytes()).is_nan()); // exponent sign, no exponent digit
        assert!(is_number("1e5e5".as_bytes()).is_nan()); // two exponents
        assert!(is_number("1.2.3".as_bytes()).is_nan()); // two dots
        assert!(is_number("1e5.0".as_bytes()).is_nan()); // dot after exponent

        // lone sign / dot / empty are NaN (not a silently-accepted integer)
        assert!(is_number("-".as_bytes()).is_nan());
        assert!(is_number("+".as_bytes()).is_nan());
        assert!(is_number(".".as_bytes()).is_nan());
        assert!(is_number("".as_bytes()).is_nan());
    }
}
