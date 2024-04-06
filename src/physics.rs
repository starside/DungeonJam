pub fn wrap_double_norm(val: f64) -> f64 {
    (val % 1.0).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_double() {
        assert_eq!(wrap_double_norm(0.0), 0.0);
        assert_eq!(wrap_double_norm(1.0), 0.0);
        assert_eq!(wrap_double_norm(6.5), 0.5);
        assert_eq!(wrap_double_norm(-6.5), 0.5);
    }
}
