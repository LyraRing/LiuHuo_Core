pub mod config;
pub mod defs;
pub mod raw_defs;
pub mod r#type;
pub mod utility;

use std::collections::HashMap;

pub type Tags = HashMap<String, String>;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LiuHuoConfig {}

#[cfg(test)]
mod tests {
    use crate::utility::parse_string2int;

    #[test]
    fn parse_string2int_supports_signed_literals() {
        assert_eq!(parse_string2int::<i32>("-42").unwrap(), -42);
        assert_eq!(parse_string2int::<i32>("-0x10").unwrap(), -16);
        assert_eq!(parse_string2int::<i32>("+0b1010").unwrap(), 10);
        assert_eq!(parse_string2int::<i32>("--0o7").unwrap(), 7);
    }

    #[test]
    fn parse_string2int_supports_chained_shifts_left_to_right() {
        assert_eq!(parse_string2int::<i32>("1 << 4 >> 1").unwrap(), 8);
        assert_eq!(parse_string2int::<i32>("1 << 2 << 3").unwrap(), 32);
        assert_eq!(parse_string2int::<i32>("64 >> 2 >> 1").unwrap(), 8);
    }

    #[test]
    fn parse_string2int_supports_signed_shift_operands() {
        assert_eq!(parse_string2int::<i32>("-0x2 << 3").unwrap(), -16);
        assert_eq!(parse_string2int::<i32>("8 >> +2").unwrap(), 2);
    }

    #[test]
    fn parse_string2int_rejects_invalid_shift_counts() {
        assert!(parse_string2int::<i32>("1 << -1").is_err());
        assert!(parse_string2int::<i32>("1 << 1 <<").is_err());
    }

    #[test]
    fn parse_string2int_checks_type_overflow() {
        assert_eq!(parse_string2int::<u8>("255").unwrap(), 255);
        assert_eq!(parse_string2int::<i8>("-0x80").unwrap(), -128);
        assert!(parse_string2int::<u8>("256").is_err());
        assert!(parse_string2int::<i8>("128").is_err());
        assert!(parse_string2int::<u8>("-1").is_err());
        assert!(parse_string2int::<u8>("1 << 8").is_err());
    }
}
