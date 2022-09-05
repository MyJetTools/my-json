const NULL_LC: [u8; 4] = [b'n', b'u', b'l', b'l'];
const NULL_UC: [u8; 4] = [b'N', b'U', b'L', b'L'];

pub fn is_null(src: &[u8]) -> bool {
    if is_that_value(&NULL_LC, &NULL_UC, src) {
        return true;
    }

    return false;
}

pub fn is_number(src: &[u8]) -> bool {
    let mut dots = 0;

    let mut es = 0;

    for i in 0..src.len() {
        if src[i] == '.' as u8 {
            dots += 1;

            if dots > 1 {
                return false;
            }
            continue;
        }

        if src[i] == 'E' as u8 {
            es += 1;

            if es > 1 {
                return false;
            }

            continue;
        }

        if src[i] == b'-' {
            if i == 0 {
                continue;
            }
            return false;
        }

        if src[i] == b'+' {
            if i == 0 {
                continue;
            }
            return false;
        }

        if !(src[i] >= '0' as u8 && src[i] <= '9' as u8) {
            return false;
        }
    }

    return true;
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

pub fn is_bool(src: &[u8]) -> Option<bool> {
    if is_that_value(&TRUE_LC, &TRUE_UC, src) {
        return Some(true);
    }

    if is_that_value(&FALSE_LC, &FALSE_UC, src) {
        return Some(true);
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
        assert!(is_number("15.5".as_bytes()));

        assert!(is_number("15".as_bytes()));

        assert!(is_number("+15.5".as_bytes()));

        assert!(is_number("-15.5".as_bytes()));

        assert!(!is_number("10.0.0.3:5125".as_bytes()));
    }
}
