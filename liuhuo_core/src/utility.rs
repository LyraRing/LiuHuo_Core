pub mod type_uti;

pub use type_uti::*;

/// 宏规则，用于为有符号整数类型实现 `ParseStringInt` trait。
macro_rules! impl_parse_string_int_signed {
    ($($ty:ty),* $(,)?) => {
        $(
            impl ParseStringInt for $ty {
                fn parse_token(token: &str, is_negative: bool, source: &str) -> Result<Self, String> {
                    let (radix, digits, kind) = split_radix(token);
                    let signed_digits = if is_negative {
                        format!("-{}", digits)
                    } else {
                        digits.to_string()
                    };

                    <$ty>::from_str_radix(&signed_digits, radix)
                        .map_err(|e| format!("无法解析{}整数或发生溢出: {} ({})", kind, source, e))
                }

                fn checked_shl_value(self, shift: u32) -> Option<Self> {
                    self.checked_shl(shift)
                }

                fn checked_shr_value(self, shift: u32) -> Option<Self> {
                    self.checked_shr(shift)
                }
            }
        )*
    };
}

/// 宏规则，用于为无符号整数类型实现 `ParseStringInt` trait。
macro_rules! impl_parse_string_int_unsigned {
    ($($ty:ty),* $(,)?) => {
        $(
            impl ParseStringInt for $ty {
                fn parse_token(token: &str, is_negative: bool, source: &str) -> Result<Self, String> {
                    if is_negative {
                        return Err(format!("无符号整数不支持负数: {}", source));
                    }

                    let (radix, digits, kind) = split_radix(token);
                    <$ty>::from_str_radix(digits, radix)
                        .map_err(|e| format!("无法解析{}整数或发生溢出: {} ({})", kind, source, e))
                }

                fn checked_shl_value(self, shift: u32) -> Option<Self> {
                    self.checked_shl(shift)
                }

                fn checked_shr_value(self, shift: u32) -> Option<Self> {
                    self.checked_shr(shift)
                }
            }
        )*
    };
}

pub trait ParseStringInt: Sized + Copy {
    /// 解析单个整数 token，支持十进制、十六进制、二进制、八进制。
    fn parse_token(token: &str, is_negative: bool, source: &str) -> Result<Self, String>;

    /// 检查左移操作是否会溢出，如果不会溢出则返回 Some(结果)，否则返回 None。
    fn checked_shl_value(self, shift: u32) -> Option<Self>;

    /// 检查右移操作是否会溢出，如果不会溢出则返回 Some(结果)，否则返回 None。
    fn checked_shr_value(self, shift: u32) -> Option<Self>;
}

impl_parse_string_int_signed!(i8, i16, i32, i64, i128, isize);
impl_parse_string_int_unsigned!(u8, u16, u32, u64, u128, usize);

/// 将字符串解析为整数，支持十进制、十六进制、二进制、八进制以及移位操作，数字可以使用下划线分割。
pub fn parse_string2int<T>(value: &str) -> Result<T, String>
where
    T: ParseStringInt,
{
    // 移除下划线并去除前后空白字符，同时将字符串转换为小写，便于简化后续操作
    let value = value.replace("_", "").trim().to_lowercase();
    if value.is_empty() {
        return Err("无法解析空字符串为整数".to_string());
    }

    let mut cursor = 0;
    let mut result = parse_signed_int_token::<T>(&value, &mut cursor)?;

    // 处理移位操作，支持连续的移位操作，例如 "1 << 4 >> 1"
    loop {
        skip_spaces(&value, &mut cursor);
        if cursor >= value.len() {
            break;
        }

        let op = if value[cursor..].starts_with("<<") {
            cursor += 2;
            ShiftOp::Left
        } else if value[cursor..].starts_with(">>") {
            cursor += 2;
            ShiftOp::Right
        } else {
            return Err(format!("无效的移位操作符: {}", value));
        };

        let shift = parse_signed_int_token::<u32>(&value, &mut cursor)?;

        result = match op {
            ShiftOp::Left => result
                .checked_shl_value(shift)
                .ok_or_else(|| format!("左移溢出: {}", value))?,
            ShiftOp::Right => result
                .checked_shr_value(shift)
                .ok_or_else(|| format!("右移溢出: {}", value))?,
        };
    }

    Ok(result)
}

/// 将字符串解析为布尔值，支持 "true", "false", "1", "0", "yes", "no", "on", "off" 等表示。
pub fn parse_string2bool(value: &str) -> Result<bool, String> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(format!("无法解析布尔值: {}", value)),
    }
}

/// 移位操作符枚举，表示左移和右移。
enum ShiftOp {
    Left,
    Right,
}

/// 解析带符号的整数 token，支持十进制、十六进制、二进制、八进制。
fn parse_signed_int_token<T>(value: &str, cursor: &mut usize) -> Result<T, String>
where
    T: ParseStringInt,
{
    skip_spaces(value, cursor);
    if *cursor >= value.len() {
        return Err(format!("没有找到整数 token: {}", value));
    }

    let bytes = value.as_bytes();
    let mut is_negative = false;
    // 处理连续的符号，例如 "--0x10" 或 "++0b1010"
    while *cursor < value.len() {
        match bytes[*cursor] {
            b'+' => *cursor += 1,
            b'-' => {
                is_negative = !is_negative;
                *cursor += 1;
            }
            _ => break,
        }
        skip_spaces(value, cursor);
    }

    if *cursor >= value.len() {
        return Err(format!("没有找到整数 token: {}", value));
    }

    let token_start = *cursor; // 记录 token 的起始位置
    while *cursor < value.len() {
        let remaining = &value[*cursor..];
        // 如果遇到移位操作符，说明当前 token 已经结束
        if remaining.starts_with("<<") || remaining.starts_with(">>") {
            break;
        }

        // 如果遇到空白字符，检查后续是否是移位操作符，如果是，则当前 token 结束
        if bytes[*cursor].is_ascii_whitespace() {
            let mut lookahead = *cursor;
            skip_spaces(value, &mut lookahead);
            if lookahead < value.len()
                && (value[lookahead..].starts_with("<<") || value[lookahead..].starts_with(">>"))
            {
                *cursor = lookahead;
                break;
            }
        }

        *cursor += 1;
    }

    let token = value[token_start..*cursor].trim(); // 提取 token，并去除前后空白字符
    if token.is_empty() {
        return Err(format!("没有找到整数 token: {}", value));
    }

    T::parse_token(token, is_negative, value)
}

/// 将字符串解析为整数，支持十进制、十六进制、二进制、八进制。
fn split_radix(token: &str) -> (u32, &str, &'static str) {
    if let Some(rest) = token.strip_prefix("0x") {
        (16, rest, "十六进制")
    } else if let Some(rest) = token.strip_prefix("0b") {
        (2, rest, "二进制")
    } else if let Some(rest) = token.strip_prefix("0o") {
        (8, rest, "八进制")
    } else {
        (10, token, "十进制")
    }
}

/// 跳过字符串中的空白字符，更新光标位置。
fn skip_spaces(value: &str, cursor: &mut usize) {
    let bytes = value.as_bytes();
    while *cursor < value.len() && bytes[*cursor].is_ascii_whitespace() {
        *cursor += 1;
    }
}
