pub fn wrap_double_norm(val: f64) -> f64 {
    let range = 1.0;
    let val = if val < 0.0 { val + range } else { val };
    val % range
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_double() {
        assert_eq!(wrap_double_norm(0.0), 0.0);
        assert_eq!(wrap_double_norm(1.0), 0.0);
        assert_eq!(wrap_double_norm(2.5), 0.5);
    }
}
