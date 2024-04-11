pub fn wrap_double_norm(val: f64) -> f64 {
    if val < 0.0 {
        1.0 - (val % 1.0).abs()
    } else {
        (val % 1.0).abs()
    }
}

pub fn wrap_double_range(val: f64, range: f64) -> f64 {
    if val < 0.0 {
        range - (val % range).abs()
    } else {
        (val % range).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_double() {
        assert_eq!(wrap_double_norm(0.0), 0.0);
        assert_eq!(wrap_double_norm(1.0), 0.0);
        assert_eq!(wrap_double_norm(6.5), 0.5);
        assert_eq!(wrap_double_norm(-6.4), 0.6);
    }
}
