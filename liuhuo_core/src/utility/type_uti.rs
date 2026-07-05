use std::collections::HashMap;

use crate::{
    Tags,
    defs::{BeanId, EnumId},
    r#type::{TAG_KEY_RANGE, TypeKind, TypeMeta, ValidatorRule},
};

/// 从字符串中解析类型名称及标签（`@k1=v1|k2=v2` 格式）。
///
/// # 支持的格式
///
/// ## 基本类型
/// - `bool` / `Bool`
/// - `i32` / `int32` / `int` / `I32` / `Int32` / `Int`
/// - `i64` / `int64` / `I64` / `Int64`
/// - `f32` / `float` / `F32` / `Float`
/// - `f64` / `double` / `F64` / `Double`
/// - `string` / `String`
///
/// ## 枚举类型
/// - `A.B.C.Color` — 完整路径的枚举名，需在 `name2enum` 中注册
/// - `Color` — 简单枚举名，需在 `name2enum` 中注册
///
/// ## Bean 类型
/// - `A.B.C.Skill` — 完整路径的 Bean 名，需在 `name2bean` 中注册
/// - `Skill` — 简单 Bean 名，需在 `name2bean` 中注册
///
/// ## 集合类型
/// - `array<T>` / `Array<T>` — 数组（一维），等价于 `array<T,1>`
/// - `array<T,dim>` / `Array<T,dim>` — 数组（指定维度）
/// - `list<T>` / `List<T>` — 列表
/// - `set<T>` / `Set<T>` — 集合
/// - `map<K,V>` / `Map<K,V>` — 映射
/// - `matrix<T,dim1,dim2>` / `Matrix<T,dim1,dim2>` — dim1行×dim2列的矩阵
///
/// ## 可空类型
/// - 在类型后加 `?` 后缀表示可空，例如 `int?`、`string?`、`array<int>?`
///
/// ## 标签
/// - 标签以 `@` 开头，直接附加在类型名后（可空后缀之后），无空格分隔
/// - 多个标签用 `|` 分隔：`int?@range=0..|label=age`
/// - 标签也可以出现在泛型参数内部，例如 `Map<int@range=0.., List<string>>`
///
/// # 参数
/// - `s`: 类型字符串
/// - `name2enum`: 枚举全名到 EnumId 的映射
/// - `name2bean`: Bean 全名到 BeanId 的映射
///
/// # 返回值
/// - `Ok(TypeKind)`: 解析成功
/// - `Err(String)`: 解析失败，包含错误信息
pub fn parse_type(
    s: &str,
    name2enum: &HashMap<String, EnumId>,
    name2bean: &HashMap<String, BeanId>,
) -> Result<TypeKind, String> {
    parse_inner_type(s.trim(), name2enum, name2bean)
}

/// 从 Tags 中解析 ValidatorRule 列表。支持
/// - `range` 标签，格式为 `min..max`、`min..` 或 `..max`，表示整数范围验证规则。
/// - `rangef` 标签，格式为 `min..max`、`min..` 或 `..max`，表示浮点数范围验证规则。
/// - `len` 标签，格式为 `int` 或 `min..max`，表示容器长度验证规则。
pub fn parse_validator_rules(tags: &Tags) -> Result<Vec<ValidatorRule>, String> {
    let mut rules = Vec::new();

    // 解析 range 标签（整数范围）
    if let Some(range_str) = tags.get(TAG_KEY_RANGE) {
        let range_rule = parse_validator_rules_range_int_str(range_str)?;
        rules.push(range_rule);
    }

    // 解析 rangef 标签（浮点数范围）
    if let Some(range_str) = tags.get("rangef") {
        let range_rule = parse_validator_rules_range_float_str(range_str)?;
        rules.push(range_rule);
    }

    // 解析 len 标签（长度验证）
    if let Some(len_str) = tags.get("len") {
        let len_rule = parse_validator_rules_len_str(len_str)?;
        rules.push(len_rule);
    }

    Ok(rules)
}

/// 解析整数范围字符串，返回 Range rule
pub fn parse_validator_rules_range_int_str(s: &str) -> Result<ValidatorRule, String> {
    let (min, max) = parse_range(s)?;
    Ok(ValidatorRule::Range { min, max })
}

/// 解析浮点数范围字符串，返回 RangeF rule
pub fn parse_validator_rules_range_float_str(s: &str) -> Result<ValidatorRule, String> {
    let (min, max) = parse_range(s)?;
    Ok(ValidatorRule::RangeF { min, max })
}

/// 解析长度验证字符串，返回 Length rule
/// 支持格式：
/// - "5" -> min=Some(5), max=Some(5) (精确长度)
/// - "1..100" -> min=Some(1), max=Some(100) (长度范围)
pub fn parse_validator_rules_len_str(s: &str) -> Result<ValidatorRule, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("长度字符串不能为空".to_string());
    }

    // 检查是否包含 ".." (范围格式)
    if s.find("..").is_some() {
        let (min, max) = parse_range(s)?;
        Ok(ValidatorRule::Length { min, max })
    } else {
        // 精确长度格式（单个整数）
        let len = s
            .parse::<usize>()
            .map_err(|_| format!("无法解析长度值: '{}'", s))?;

        Ok(ValidatorRule::Length {
            min: Some(len),
            max: Some(len),
        })
    }
}

fn parse_range<T>(s: &str) -> Result<(Option<T>, Option<T>), String>
where
    T: std::str::FromStr + std::cmp::PartialOrd + std::fmt::Display + Copy,
    T::Err: std::fmt::Display,
{
    let s = s.trim();
    if s.is_empty() {
        return Err("范围字符串不能为空".to_string());
    }

    // 查找 ".." 分隔符
    let dotdot_pos = s.find("..").ok_or_else(|| {
        format!(
            "范围格式无效: '{}'，应为 'min..max'、'min..' 或 '..max' 格式",
            s
        )
    })?;

    let left_part = s[..dotdot_pos].trim();
    let right_part = s[dotdot_pos + 2..].trim();

    // 解析最小值
    let min = if left_part.is_empty() {
        None
    } else {
        let min_val = left_part
            .parse::<T>()
            .map_err(|_| format!("无法解析范围最小值: '{}'", left_part))?;
        Some(min_val)
    };

    // 解析最大值
    let max = if right_part.is_empty() {
        None
    } else {
        let max_val = right_part
            .parse::<T>()
            .map_err(|_| format!("无法解析范围最大值: '{}'", right_part))?;
        Some(max_val)
    };

    // 至少需要一个边界
    if min.is_none() && max.is_none() {
        return Err(format!(
            "范围无效: '{}'，至少需要指定一个边界（最小值或最大值）",
            s
        ));
    }

    // 如果两边都有值，验证左边小于右边
    if let (Some(min_val), Some(max_val)) = (min, max) {
        if min_val >= max_val {
            return Err(format!(
                "范围无效: 最小值 {} 必须小于最大值 {}",
                min_val, max_val
            ));
        }
    }

    Ok((min, max))
}

// pub fn parse_vadidator_rules_range(tags: &Tags) -> Result<ValidatorRule, String> {
//     let range_str = tags
//         .get(TAG_KEY_RANGE)
//         .ok_or(format!("缺少 {} 标签", TAG_KEY_RANGE))?;

//     let range_rule = ValidatorRule::Range(range_str.clone());
//     Ok(range_rule)
// }

/// 递归解析一个完整的类型表达式（包括标签、可空、泛型嵌套）。
///
/// 解析顺序（以 `int?@range=0..|label=age` 为例）：
/// 1. 从末尾提取标签：`@range=0..|label=age` → `("int?", tags)`
/// 2. 从剩余部分提取可空：`?` → `("int", nullable)`
/// 3. 解析基础类型表达式：`"int"` → `TypeKind::I32`
/// 4. 将 nullable 和 tags 应用到 meta 上
///
/// 对于泛型类型（如 `List<int?>`），`parse_type_expr` 会递归调用 `parse_inner_type`
/// 来解析内部类型（如 `int?`），形成完整的递归链路。
fn parse_inner_type(
    s: &str,
    name2enum: &HashMap<String, EnumId>,
    name2bean: &HashMap<String, BeanId>,
) -> Result<TypeKind, String> {
    // Step 1: 提取后置标签（@k1=v1|k2=v2 附加在类型名后）
    let (type_str, tags) = parse_trailing_tags(s)?;

    // Step 2: 检查可空后缀（?）
    let (type_str, nullable) = if let Some(rest) = type_str.strip_suffix('?') {
        (rest.trim(), true)
    } else {
        (type_str, false)
    };

    // Step 3: 递归解析类型表达式（泛型内部会再次调用 parse_inner_type）
    let type_kind = parse_type_expr(type_str, name2enum, name2bean)?;

    // Step 4: 对支持 meta 的类型应用 nullable 和 tags
    Ok(apply_meta(type_kind, nullable, tags))
}

/// 从类型名末尾提取后置标签（`@k1=v1|k2=v2` 格式，无空格分隔）。
/// 例如 `int@range=0..` 返回 `("int", {"range": "0.."})`。
/// 例如 `int?@range=0..|label=age` 返回 `("int?", {"range": "0..", "label": "age"})`。
/// 如果末尾没有标签，返回 `(s, {})`。
///
/// 注意：只有当 `@` 不在尖括号内部时才进行解析（避免将泛型内的类型注解误判）。
fn parse_trailing_tags(s: &str) -> Result<(&str, Tags), String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok((s, Tags::new()));
    }

    // 查找最后一个 '@'，且确保它不在尖括号内部
    let mut depth = 0u32;
    let mut last_at_pos: Option<usize> = None;

    for (i, ch) in s.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            '@' if depth == 0 => {
                // 只记录不在尖括号内部的 '@'
                // 同时检查 @ 不在开头（开头是前置标签，我们不支持）
                if i > 0 {
                    last_at_pos = Some(i);
                }
            }
            _ => {}
        }
    }

    if let Some(at_pos) = last_at_pos {
        let tag_section = &s[at_pos + 1..];
        if tag_section.is_empty() {
            return Err(format!("标签格式无效: '{}'，@ 后缺少标签内容", s));
        }

        let mut tags = Tags::new();

        // 用 '|' 分割多个 k=v 对
        for part in tag_section.split('|') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let eq_pos = part
                .find('=')
                .ok_or_else(|| format!("标签格式无效: '{}'，应为 @k1=v1|k2=v2 格式", s))?;

            let key = part[..eq_pos].trim();
            if key.is_empty() {
                return Err(format!("标签键不能为空: '{}'", s));
            }
            let value = part[eq_pos + 1..].trim();

            if tags.insert(key.to_string(), value.to_string()).is_some() {
                return Err(format!("重复的标签键: '{}'", key));
            }
        }

        let type_name = s[..at_pos].trim();
        return Ok((type_name, tags));
    }

    Ok((s, Tags::new()))
}

/// 解析类型表达式字符串，返回 TypeKind。
/// 对于泛型集合类型，内部元素递归调用 `parse_inner_type` 以支持标签和可空。
fn parse_type_expr(
    s: &str,
    name2enum: &HashMap<String, EnumId>,
    name2bean: &HashMap<String, BeanId>,
) -> Result<TypeKind, String> {
    let s = s.trim();

    if s.is_empty() {
        return Err("类型表达式不能为空".to_string());
    }

    // 检查集合类型：array<T>、array<T,dim>、list<T>、set<T>、map<K,V>、matrix<T,dim1,dim2>
    // 泛型参数内部的类型递归调用 parse_inner_type 以支持标签和可空
    if let Some(inner) = try_extract_generic(s, "array") {
        let params = split_generic_params(inner)?;
        let elem_type = parse_inner_type(params[0], name2enum, name2bean)?;
        let dimensions: usize = if params.len() >= 2 {
            parse_dimension(params[1])?
        } else {
            1
        };
        if params.len() > 2 {
            return Err(format!(
                "array 最多接受 2 个参数 (类型, 维度)，但得到了 {} 个",
                params.len()
            ));
        }
        return Ok(TypeKind::Array {
            elem_type: Box::new(elem_type),
            dimensions,
        });
    }

    if let Some(inner) = try_extract_generic(s, "matrix") {
        let params = split_generic_params(inner)?;
        if params.len() != 3 {
            return Err(format!(
                "Matrix 需要三个参数 (类型, 行数, 列数)，但得到了 {} 个",
                params.len()
            ));
        }
        let elem_type = parse_inner_type(params[0], name2enum, name2bean)?;
        let rows = parse_dimension(params[1])?;
        let cols = parse_dimension(params[2])?;
        return Ok(TypeKind::Matrix {
            elem_type: Box::new(elem_type),
            dimensions: (rows, cols),
        });
    }

    if let Some(inner) = try_extract_generic(s, "list") {
        let elem_type = parse_inner_type(inner, name2enum, name2bean)?;
        return Ok(TypeKind::List {
            elem_type: Box::new(elem_type),
        });
    }

    if let Some(inner) = try_extract_generic(s, "set") {
        let elem_type = parse_inner_type(inner, name2enum, name2bean)?;
        return Ok(TypeKind::Set {
            elem_type: Box::new(elem_type),
        });
    }

    if let Some(inner) = try_extract_generic(s, "map") {
        // Map 有两个类型参数，用逗号分隔
        let params = split_generic_params(inner)?;
        if params.len() != 2 {
            return Err(format!(
                "Map 需要两个类型参数，但得到了 {} 个",
                params.len()
            ));
        }
        let key_type = parse_inner_type(params[0], name2enum, name2bean)?;
        let value_type = parse_inner_type(params[1], name2enum, name2bean)?;
        return Ok(TypeKind::Map {
            key_type: Box::new(key_type),
            value_type: Box::new(value_type),
        });
    }

    // 基本类型（大小写不敏感）
    let lower = s.to_lowercase();
    match lower.as_str() {
        "bool" => {
            return Ok(TypeKind::Bool(TypeMeta {
                nullable: false,
                tags: Tags::new(),
                validator_rules: vec![],
            }));
        }
        "i32" | "int32" | "int" => {
            return Ok(TypeKind::I32(TypeMeta {
                nullable: false,
                tags: Tags::new(),
                validator_rules: vec![],
            }));
        }
        "i64" | "int64" => {
            return Ok(TypeKind::I64(TypeMeta {
                nullable: false,
                tags: Tags::new(),
                validator_rules: vec![],
            }));
        }
        "f32" | "float" => {
            return Ok(TypeKind::F32(TypeMeta {
                nullable: false,
                tags: Tags::new(),
                validator_rules: vec![],
            }));
        }
        "f64" | "double" => {
            return Ok(TypeKind::F64(TypeMeta {
                nullable: false,
                tags: Tags::new(),
                validator_rules: vec![],
            }));
        }
        "string" => {
            return Ok(TypeKind::String(TypeMeta {
                nullable: false,
                tags: Tags::new(),
                validator_rules: vec![],
            }));
        }
        _ => {}
    }

    // 检查是否是枚举或 Bean（带点号的完整路径或简单名称）
    // 优先尝试完整匹配
    if let Some(&enum_id) = name2enum.get(s) {
        return Ok(TypeKind::Enum {
            meta: TypeMeta {
                nullable: false,
                tags: Tags::new(),
                validator_rules: vec![],
            },
            def: enum_id,
        });
    }
    if let Some(&bean_id) = name2bean.get(s) {
        return Ok(TypeKind::Bean {
            meta: TypeMeta {
                nullable: false,
                tags: Tags::new(),
                validator_rules: vec![],
            },
            def: bean_id,
        });
    }

    // 如果包含点号但未找到，给出更明确的错误
    if s.contains('.') {
        return Err(format!(
            "未找到类型: '{}'，既不是已注册的枚举也不是已注册的 Bean",
            s
        ));
    }

    Err(format!("未知的类型: '{}'", s))
}

/// 尝试从字符串中提取泛型参数。
/// 例如，`array<int>` 返回 `Some("int")`，`Array<int>` 也返回 `Some("int")`。
/// 如果字符串不以指定的类型名称开头，或没有尖括号，返回 `None`。
fn try_extract_generic<'a>(s: &'a str, type_name: &str) -> Option<&'a str> {
    let s = s.trim();
    let name_len = type_name.len();

    if s.len() <= name_len + 2 {
        // 至少需要 `type<>` 的长度
        return None;
    }

    // 大小写不敏感地检查前缀
    let prefix = &s[..name_len];
    if !prefix.eq_ignore_ascii_case(type_name) {
        return None;
    }

    let rest = s[name_len..].trim();
    if !rest.starts_with('<') {
        return None;
    }

    // 找到匹配的右尖括号
    let inner = find_matching_angle(rest)?;
    Some(inner)
}

/// 在字符串中查找匹配的尖括号对，返回尖括号内的内容。
/// 输入应以 `<` 开头。
fn find_matching_angle(s: &str) -> Option<&str> {
    let s = s.trim();
    if !s.starts_with('<') {
        return None;
    }

    let mut depth = 0u32;
    for (i, ch) in s.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    // i 是 '>' 的位置，返回从 1 到 i 的内容
                    return Some(&s[1..i]);
                }
            }
            _ => {}
        }
    }

    None // 没有匹配的右尖括号
}

/// 分割泛型参数列表。
/// 例如，`int, 3` 返回 `["int", "3"]`。
/// 例如，`string, array<int>` 返回 `["string", "array<int>"]`。
/// 注意逗号可能在嵌套的尖括号内，需要正确处理。
fn split_generic_params(s: &str) -> Result<Vec<&str>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("泛型参数列表不能为空".to_string());
    }

    let mut params: Vec<&str> = Vec::new();
    let mut depth = 0u32;
    let mut last_split = 0usize;

    for (i, ch) in s.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            ',' if depth == 0 => {
                let param = s[last_split..i].trim();
                if param.is_empty() {
                    return Err(format!("泛型参数列表中有空的参数: '{}'", s));
                }
                params.push(param);
                last_split = i + 1;
            }
            _ => {}
        }
    }

    // 最后一个参数
    let last_param = s[last_split..].trim();
    if last_param.is_empty() {
        return Err(format!("泛型参数列表以逗号结尾: '{}'", s));
    }
    params.push(last_param);

    if params.is_empty() {
        return Err(format!("泛型参数列表为空: '{}'", s));
    }

    Ok(params)
}

/// 将字符串解析为维度数值（正整数）。
fn parse_dimension(s: &str) -> Result<usize, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("维度值不能为空".to_string());
    }
    let value: usize = s
        .parse()
        .map_err(|_| format!("无法解析维度值: '{}'，应为正整数", s))?;
    if value == 0 {
        return Err(format!("维度值必须大于 0，但得到了: '{}'", s));
    }
    Ok(value)
}

/// 对支持 meta 的 TypeKind 应用 nullable 和 tags。
/// 集合类型（Array、List、Set、Map、Matrix）没有 meta 字段，直接返回。
fn apply_meta(type_kind: TypeKind, nullable: bool, tags: Tags) -> TypeKind {
    match type_kind {
        TypeKind::Bool(meta) => TypeKind::Bool(TypeMeta {
            nullable,
            tags,
            ..meta
        }),
        TypeKind::I32(meta) => TypeKind::I32(TypeMeta {
            nullable,
            tags,
            ..meta
        }),
        TypeKind::I64(meta) => TypeKind::I64(TypeMeta {
            nullable,
            tags,
            ..meta
        }),
        TypeKind::F32(meta) => TypeKind::F32(TypeMeta {
            nullable,
            tags,
            ..meta
        }),
        TypeKind::F64(meta) => TypeKind::F64(TypeMeta {
            nullable,
            tags,
            ..meta
        }),
        TypeKind::String(meta) => TypeKind::String(TypeMeta {
            nullable,
            tags,
            ..meta
        }),
        TypeKind::Enum { meta, def } => TypeKind::Enum {
            meta: TypeMeta {
                nullable,
                tags,
                ..meta
            },
            def,
        },
        TypeKind::Bean { meta, def } => TypeKind::Bean {
            meta: TypeMeta {
                nullable,
                tags,
                ..meta
            },
            def,
        },
        // 集合类型没有 meta，直接返回
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#type::TypeKind;

    fn empty_enum_map() -> HashMap<String, EnumId> {
        HashMap::new()
    }

    fn empty_bean_map() -> HashMap<String, BeanId> {
        HashMap::new()
    }

    // ===== 基本类型测试 =====

    #[test]
    fn test_parse_bool() {
        let result = parse_type("bool", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_bool] parse_type(\"bool\") = {:?}", result);
        println!("  -> 期望: TypeKind::Bool(_)");
    }

    #[test]
    fn test_parse_bool_case_insensitive() {
        let result = parse_type("Bool", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!(
            "[test_parse_bool_case_insensitive] parse_type(\"Bool\") = {:?}",
            result
        );
        println!("  -> 期望: TypeKind::Bool(_)");
    }

    #[test]
    fn test_parse_i32() {
        let result = parse_type("i32", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_i32] parse_type(\"i32\") = {:?}", result);
        println!("  -> 期望: TypeKind::I32(_)");
    }

    #[test]
    fn test_parse_int32() {
        let result = parse_type("int32", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_int32] parse_type(\"int32\") = {:?}", result);
        println!("  -> 期望: TypeKind::I32(_)");
    }

    #[test]
    fn test_parse_int() {
        let result = parse_type("int", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_int] parse_type(\"int\") = {:?}", result);
        println!("  -> 期望: TypeKind::I32(_)");
    }

    #[test]
    fn test_parse_i64() {
        let result = parse_type("i64", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_i64] parse_type(\"i64\") = {:?}", result);
        println!("  -> 期望: TypeKind::I64(_)");
    }

    #[test]
    fn test_parse_int64() {
        let result = parse_type("int64", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_int64] parse_type(\"int64\") = {:?}", result);
        println!("  -> 期望: TypeKind::I64(_)");
    }

    #[test]
    fn test_parse_f32() {
        let result = parse_type("f32", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_f32] parse_type(\"f32\") = {:?}", result);
        println!("  -> 期望: TypeKind::F32(_)");
    }

    #[test]
    fn test_parse_float() {
        let result = parse_type("float", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_float] parse_type(\"float\") = {:?}", result);
        println!("  -> 期望: TypeKind::F32(_)");
    }

    #[test]
    fn test_parse_f64() {
        let result = parse_type("f64", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_f64] parse_type(\"f64\") = {:?}", result);
        println!("  -> 期望: TypeKind::F64(_)");
    }

    #[test]
    fn test_parse_double() {
        let result = parse_type("double", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_double] parse_type(\"double\") = {:?}", result);
        println!("  -> 期望: TypeKind::F64(_)");
    }

    #[test]
    fn test_parse_string() {
        let result = parse_type("string", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!("[test_parse_string] parse_type(\"string\") = {:?}", result);
        println!("  -> 期望: TypeKind::String(_)");
    }

    #[test]
    fn test_parse_string_case_insensitive() {
        let result = parse_type("String", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!(
            "[test_parse_string_case_insensitive] parse_type(\"String\") = {:?}",
            result
        );
        println!("  -> 期望: TypeKind::String(_)");
    }

    // ===== 可空类型测试 =====

    #[test]
    fn test_parse_nullable_int() {
        let result = parse_type("int?", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::I32(meta) => {
                println!(
                    "[test_parse_nullable_int] parse_type(\"int?\") = {:?}",
                    result
                );
                println!("  -> nullable = {}, 期望: true", meta.nullable);
            }
            _ => {
                println!(
                    "[test_parse_nullable_int] parse_type(\"int?\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::I32(meta)");
            }
        }
    }

    #[test]
    fn test_parse_nullable_string() {
        let result = parse_type("string?", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::String(meta) => {
                println!(
                    "[test_parse_nullable_string] parse_type(\"string?\") = {:?}",
                    result
                );
                println!("  -> nullable = {}, 期望: true", meta.nullable);
            }
            _ => {
                println!(
                    "[test_parse_nullable_string] parse_type(\"string?\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::String(meta)");
            }
        }
    }

    // ===== 集合类型测试 =====

    #[test]
    fn test_parse_array() {
        let result = parse_type("array<int>", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::Array {
                elem_type,
                dimensions,
            } => {
                println!(
                    "[test_parse_array] parse_type(\"array<int>\") = Array {{ elem_type: {:?}, dimensions: {} }}",
                    elem_type, dimensions
                );
                println!("  -> dimensions = {}, 期望: 1", dimensions);
                println!("  -> elem_type 期望: I32(_)");
            }
            _ => {
                println!(
                    "[test_parse_array] parse_type(\"array<int>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Array {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_array_case_insensitive() {
        let result = parse_type("Array<int>", &empty_enum_map(), &empty_bean_map()).unwrap();
        println!(
            "[test_parse_array_case_insensitive] parse_type(\"Array<int>\") = {:?}",
            result
        );
        println!("  -> 期望: TypeKind::Array {{ .. }}");
    }

    #[test]
    fn test_parse_list() {
        let result = parse_type("list<string>", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::List { elem_type } => {
                println!(
                    "[test_parse_list] parse_type(\"list<string>\") = List {{ elem_type: {:?} }}",
                    elem_type
                );
                println!("  -> elem_type 期望: String(_)");
            }
            _ => {
                println!(
                    "[test_parse_list] parse_type(\"list<string>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::List {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_set() {
        let result = parse_type("set<bool>", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::Set { elem_type } => {
                println!(
                    "[test_parse_set] parse_type(\"set<bool>\") = Set {{ elem_type: {:?} }}",
                    elem_type
                );
                println!("  -> elem_type 期望: Bool(_)");
            }
            _ => {
                println!("[test_parse_set] parse_type(\"set<bool>\") = {:?}", result);
                println!("  -> 期望: TypeKind::Set {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_map() {
        let result = parse_type("map<string,int?>", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::Map {
                key_type,
                value_type,
            } => {
                println!(
                    "[test_parse_map] parse_type(\"map<string,int?>\") = Map {{ key_type: {:?}, value_type: {:?} }}",
                    key_type, value_type
                );
                println!("  -> key_type 期望: String(_)");
                println!("  -> value_type 期望: I32(_)");
            }
            _ => {
                println!(
                    "[test_parse_map] parse_type(\"map<string,int?>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Map {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_nested_generic() {
        let result = parse_type(
            "map<string,array<int>>",
            &empty_enum_map(),
            &empty_bean_map(),
        )
        .unwrap();
        match &result {
            TypeKind::Map {
                key_type,
                value_type,
            } => {
                println!(
                    "[test_parse_nested_generic] parse_type(\"map<string,array<int>>\") = Map {{ key_type: {:?}, value_type: {:?} }}",
                    key_type, value_type
                );
                println!("  -> key_type 期望: String(_)");
                println!("  -> value_type 期望: Array {{ .. }}");
            }
            _ => {
                println!(
                    "[test_parse_nested_generic] parse_type(\"map<string,array<int>>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Map {{ .. }}");
            }
        }
    }

    // ===== 标签测试（后置 @k=v 格式）=====

    #[test]
    fn test_parse_with_trailing_tag() {
        let result = parse_type("int@range=1..100", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::I32(meta) => {
                let tag_val = meta.tags.get("range");
                println!(
                    "[test_parse_with_trailing_tag] parse_type(\"int@range=1..100\") = I32(meta)"
                );
                println!(
                    "  -> tags[\"range\"] = {:?}, 期望: Some(\"1..100\")",
                    tag_val
                );
            }
            _ => {
                println!(
                    "[test_parse_with_trailing_tag] parse_type(\"int@range=1..100\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::I32(meta)");
            }
        }
    }

    #[test]
    fn test_parse_with_multiple_trailing_tags() {
        let result = parse_type(
            "int@range=1..100|label=age",
            &empty_enum_map(),
            &empty_bean_map(),
        )
        .unwrap();
        match &result {
            TypeKind::I32(meta) => {
                let range_val = meta.tags.get("range");
                let label_val = meta.tags.get("label");
                println!(
                    "[test_parse_with_multiple_trailing_tags] parse_type(\"int@range=1..100|label=age\") = I32(meta)"
                );
                println!(
                    "  -> tags[\"range\"] = {:?}, 期望: Some(\"1..100\")",
                    range_val
                );
                println!(
                    "  -> tags[\"label\"] = {:?}, 期望: Some(\"age\")",
                    label_val
                );
            }
            _ => {
                println!(
                    "[test_parse_with_multiple_trailing_tags] parse_type(\"int@range=1..100|label=age\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::I32(meta)");
            }
        }
    }

    #[test]
    fn test_parse_nullable_with_trailing_tag() {
        // int?@range=1..100 → nullable=true, tags={"range":"1..100"}
        let result = parse_type("int?@range=1..100", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::I32(meta) => {
                let tag_val = meta.tags.get("range");
                println!(
                    "[test_parse_nullable_with_trailing_tag] parse_type(\"int?@range=1..100\") = I32(meta)"
                );
                println!("  -> nullable = {}, 期望: true", meta.nullable);
                println!(
                    "  -> tags[\"range\"] = {:?}, 期望: Some(\"1..100\")",
                    tag_val
                );
            }
            _ => {
                println!(
                    "[test_parse_nullable_with_trailing_tag] parse_type(\"int?@range=1..100\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::I32(meta)");
            }
        }
    }

    #[test]
    fn test_parse_nullable_with_multiple_trailing_tags() {
        // int?@range=1..100|label=age → nullable=true, tags={"range":"1..100","label":"age"}
        let result = parse_type(
            "int?@range=1..100|label=age",
            &empty_enum_map(),
            &empty_bean_map(),
        )
        .unwrap();
        match &result {
            TypeKind::I32(meta) => {
                let range_val = meta.tags.get("range");
                let label_val = meta.tags.get("label");
                println!(
                    "[test_parse_nullable_with_multiple_trailing_tags] parse_type(\"int?@range=1..100|label=age\") = I32(meta)"
                );
                println!("  -> nullable = {}, 期望: true", meta.nullable);
                println!(
                    "  -> tags[\"range\"] = {:?}, 期望: Some(\"1..100\")",
                    range_val
                );
                println!(
                    "  -> tags[\"label\"] = {:?}, 期望: Some(\"age\")",
                    label_val
                );
            }
            _ => {
                println!(
                    "[test_parse_nullable_with_multiple_trailing_tags] parse_type(\"int?@range=1..100|label=age\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::I32(meta)");
            }
        }
    }

    // ===== 枚举/Bean 类型测试 =====

    #[test]
    fn test_parse_enum_by_full_name() {
        let mut name2enum = HashMap::new();
        let enum_id = EnumId::default();
        name2enum.insert("A.B.C.Color".to_string(), enum_id);

        let result = parse_type("A.B.C.Color", &name2enum, &empty_bean_map()).unwrap();
        match &result {
            TypeKind::Enum { def, .. } => {
                println!(
                    "[test_parse_enum_by_full_name] parse_type(\"A.B.C.Color\") = Enum {{ def: {:?} }}",
                    def
                );
                println!("  -> def = {:?}, 期望: {:?}", def, enum_id);
            }
            _ => {
                println!(
                    "[test_parse_enum_by_full_name] parse_type(\"A.B.C.Color\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Enum {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_bean_by_full_name() {
        let mut name2bean = HashMap::new();
        let bean_id = BeanId::default();
        name2bean.insert("A.B.C.Skill".to_string(), bean_id);

        let result = parse_type("A.B.C.Skill", &empty_enum_map(), &name2bean).unwrap();
        match &result {
            TypeKind::Bean { def, .. } => {
                println!(
                    "[test_parse_bean_by_full_name] parse_type(\"A.B.C.Skill\") = Bean {{ def: {:?} }}",
                    def
                );
                println!("  -> def = {:?}, 期望: {:?}", def, bean_id);
            }
            _ => {
                println!(
                    "[test_parse_bean_by_full_name] parse_type(\"A.B.C.Skill\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Bean {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_enum_preferred_over_bean() {
        let mut name2enum = HashMap::new();
        let mut name2bean = HashMap::new();
        let enum_id = EnumId::default();
        let bean_id = BeanId::default();
        name2enum.insert("Color".to_string(), enum_id);
        name2bean.insert("Color".to_string(), bean_id);

        // Enum 优先于 Bean
        let result = parse_type("Color", &name2enum, &name2bean).unwrap();
        match &result {
            TypeKind::Enum { def, .. } => {
                println!(
                    "[test_parse_enum_preferred_over_bean] parse_type(\"Color\") = Enum {{ def: {:?} }}",
                    def
                );
                println!(
                    "  -> def = {:?}, 期望: {:?} (Enum 应优先于 Bean)",
                    def, enum_id
                );
            }
            _ => {
                println!(
                    "[test_parse_enum_preferred_over_bean] parse_type(\"Color\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Enum {{ .. }} (Enum 应优先于 Bean)");
            }
        }
    }

    // ===== 泛型嵌套 + 标签/可空（递归链路验证）=====

    #[test]
    fn test_parse_map_with_tag_on_key() {
        let result = parse_type(
            "Map<int@range=0.., List<string>>",
            &empty_enum_map(),
            &empty_bean_map(),
        )
        .unwrap();
        match &result {
            TypeKind::Map {
                key_type,
                value_type,
            } => {
                println!(
                    "[test_parse_map_with_tag_on_key] parse_type(\"Map<int@range=0.., List<string>>\") = Map {{ key_type: {:?}, value_type: {:?} }}",
                    key_type, value_type
                );
                match &**key_type {
                    TypeKind::I32(meta) => {
                        let tag_val = meta.tags.get("range");
                        println!(
                            "  -> key tags[\"range\"] = {:?}, 期望: Some(\"0..\")",
                            tag_val
                        );
                    }
                    _ => println!("  -> key 期望: I32, 实际: {:?}", key_type),
                }
                match &**value_type {
                    TypeKind::List { elem_type } => {
                        println!("  -> value elem_type = {:?}, 期望: String(_)", elem_type);
                    }
                    _ => println!("  -> value 期望: List<String>, 实际: {:?}", value_type),
                }
            }
            _ => {
                println!(
                    "[test_parse_map_with_tag_on_key] parse_type(\"Map<int@range=0.., List<string>>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Map {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_nested_list_nullable() {
        let result = parse_type("List<List<int?>>", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::List { elem_type } => {
                println!(
                    "[test_parse_nested_list_nullable] parse_type(\"List<List<int?>>\") = List {{ elem_type: {:?} }}",
                    elem_type
                );
                match &**elem_type {
                    TypeKind::List {
                        elem_type: inner_elem,
                    } => {
                        println!("  -> 内层 List elem_type = {:?}", inner_elem);
                        match &**inner_elem {
                            TypeKind::I32(meta) => {
                                println!("  -> 最内层 nullable = {}, 期望: true", meta.nullable);
                            }
                            _ => println!("  -> 最内层 期望: I32, 实际: {:?}", inner_elem),
                        }
                    }
                    _ => println!("  -> 内层 期望: List, 实际: {:?}", elem_type),
                }
            }
            _ => {
                println!(
                    "[test_parse_nested_list_nullable] parse_type(\"List<List<int?>>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::List {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_list_nullable_element() {
        let result = parse_type("List<int?>", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::List { elem_type } => {
                println!(
                    "[test_parse_list_nullable_element] parse_type(\"List<int?>\") = List {{ elem_type: {:?} }}",
                    elem_type
                );
                match &**elem_type {
                    TypeKind::I32(meta) => {
                        println!("  -> nullable = {}, 期望: true", meta.nullable);
                    }
                    _ => println!("  -> 元素 期望: I32, 实际: {:?}", elem_type),
                }
            }
            _ => {
                println!(
                    "[test_parse_list_nullable_element] parse_type(\"List<int?>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::List {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_nested_map_with_tag_and_nullable() {
        // Map<string@format=email, List<int?>?>  → 多层递归嵌套
        let result = parse_type(
            "Map<string@format=email, List<int?>?>",
            &empty_enum_map(),
            &empty_bean_map(),
        )
        .unwrap();
        match &result {
            TypeKind::Map {
                key_type,
                value_type,
            } => {
                println!(
                    "[test_parse_nested_map_with_tag_and_nullable] parse_type(\"Map<string@format=email, List<int?>?>\") = Map {{ key_type: {:?}, value_type: {:?} }}",
                    key_type, value_type
                );
                match &**key_type {
                    TypeKind::String(meta) => {
                        let tag_val = meta.tags.get("format");
                        println!(
                            "  -> key tags[\"format\"] = {:?}, 期望: Some(\"email\")",
                            tag_val
                        );
                    }
                    _ => println!("  -> key 期望: String, 实际: {:?}", key_type),
                }
                match &**value_type {
                    TypeKind::List { elem_type } => {
                        println!("  -> value elem_type = {:?}", elem_type);
                        match &**elem_type {
                            TypeKind::I32(meta) => {
                                println!("  -> 元素 nullable = {}, 期望: true", meta.nullable);
                            }
                            _ => println!("  -> 元素 期望: I32, 实际: {:?}", elem_type),
                        }
                    }
                    _ => println!("  -> value 期望: List, 实际: {:?}", value_type),
                }
            }
            _ => {
                println!(
                    "[test_parse_nested_map_with_tag_and_nullable] parse_type(\"Map<string@format=email, List<int?>?>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Map {{ .. }}");
            }
        }
    }

    // ===== array<T,dim> 和 matrix<T,dim1,dim2> 测试 =====

    #[test]
    fn test_parse_array_with_dim() {
        // array<int,3> → 三维 int 数组
        let result = parse_type("array<int,3>", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::Array {
                elem_type,
                dimensions,
            } => {
                println!(
                    "[test_parse_array_with_dim] parse_type(\"array<int,3>\") = Array {{ elem_type: {:?}, dimensions: {} }}",
                    elem_type, dimensions
                );
                println!("  -> dimensions = {}, 期望: 3", dimensions);
                println!("  -> elem_type 期望: I32(_)");
            }
            _ => {
                println!(
                    "[test_parse_array_with_dim] parse_type(\"array<int,3>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Array {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_array_with_dim_nested() {
        // array<array<int,2>,3> → 三维数组，元素是二维 int 数组
        let result = parse_type(
            "array<array<int,2>,3>",
            &empty_enum_map(),
            &empty_bean_map(),
        )
        .unwrap();
        match &result {
            TypeKind::Array {
                elem_type,
                dimensions,
            } => {
                println!(
                    "[test_parse_array_with_dim_nested] parse_type(\"array<array<int,2>,3>\") = Array {{ elem_type: {:?}, dimensions: {} }}",
                    elem_type, dimensions
                );
                println!("  -> dimensions = {}, 期望: 3", dimensions);
                match &**elem_type {
                    TypeKind::Array {
                        elem_type: inner_elem,
                        dimensions: inner_dim,
                    } => {
                        println!(
                            "  -> 内层 Array {{ elem_type: {:?}, dimensions: {} }}",
                            inner_elem, inner_dim
                        );
                        println!("  -> 内层 dimensions = {}, 期望: 2", inner_dim);
                        println!("  -> 内层 elem_type 期望: I32(_)");
                    }
                    _ => println!("  -> 元素 期望: Array, 实际: {:?}", elem_type),
                }
            }
            _ => {
                println!(
                    "[test_parse_array_with_dim_nested] parse_type(\"array<array<int,2>,3>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Array {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_matrix() {
        // matrix<int,3,4> → 3行4列的 int 矩阵
        let result = parse_type("matrix<int,3,4>", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::Matrix {
                elem_type,
                dimensions,
            } => {
                println!(
                    "[test_parse_matrix] parse_type(\"matrix<int,3,4>\") = Matrix {{ elem_type: {:?}, dimensions: {:?} }}",
                    elem_type, dimensions
                );
                println!("  -> dimensions = {:?}, 期望: (3, 4)", dimensions);
                println!("  -> elem_type 期望: I32(_)");
            }
            _ => {
                println!(
                    "[test_parse_matrix] parse_type(\"matrix<int,3,4>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Matrix {{ .. }}");
            }
        }
    }

    #[test]
    fn test_parse_matrix_case_insensitive() {
        let result = parse_type("Matrix<float,1,1>", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::Matrix {
                elem_type,
                dimensions,
            } => {
                println!(
                    "[test_parse_matrix_case_insensitive] parse_type(\"Matrix<float,1,1>\") = Matrix {{ elem_type: {:?}, dimensions: {:?} }}",
                    elem_type, dimensions
                );
                println!("  -> dimensions = {:?}, 期望: (1, 1)", dimensions);
                println!("  -> elem_type 期望: F32(_)");
            }
            _ => {
                println!(
                    "[test_parse_matrix_case_insensitive] parse_type(\"Matrix<float,1,1>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::Matrix {{ .. }}");
            }
        }
    }

    // ===== 递归链路：泛型内部元素带可空+标签 =====

    #[test]
    fn test_parse_nested_list_nullable_with_tag() {
        // List<int?@range=0..>  → 最内层 int?@range=0..
        let result =
            parse_type("List<int?@range=0..>", &empty_enum_map(), &empty_bean_map()).unwrap();
        match &result {
            TypeKind::List { elem_type } => {
                println!(
                    "[test_parse_nested_list_nullable_with_tag] parse_type(\"List<int?@range=0..>\") = List {{ elem_type: {:?} }}",
                    elem_type
                );
                match &**elem_type {
                    TypeKind::I32(meta) => {
                        let tag_val = meta.tags.get("range");
                        println!("  -> nullable = {}, 期望: true", meta.nullable);
                        println!("  -> tags[\"range\"] = {:?}, 期望: Some(\"0..\")", tag_val);
                    }
                    _ => println!("  -> 元素 期望: I32, 实际: {:?}", elem_type),
                }
            }
            _ => {
                println!(
                    "[test_parse_nested_list_nullable_with_tag] parse_type(\"List<int?@range=0..>\") = {:?}",
                    result
                );
                println!("  -> 期望: TypeKind::List {{ .. }}");
            }
        }
    }

    // ===== Range 解析测试 =====

    #[test]
    fn test_parse_validator_rules_with_range() {
        let mut tags = Tags::new();
        tags.insert("range".to_string(), "1..100".to_string());

        let rules = parse_validator_rules(&tags).unwrap();
        assert_eq!(rules.len(), 1);

        match &rules[0] {
            ValidatorRule::Range { min, max } => {
                assert_eq!(min, &Some(1));
                assert_eq!(max, &Some(100));
            }
            _ => panic!("期望 Range 规则"),
        }
    }

    #[test]
    fn test_parse_validator_rules_range_only_min() {
        let mut tags = Tags::new();
        tags.insert("range".to_string(), "1..".to_string());

        let rules = parse_validator_rules(&tags).unwrap();

        match &rules[0] {
            ValidatorRule::Range { min, max } => {
                assert_eq!(min, &Some(1));
                assert_eq!(max, &None);
            }
            _ => panic!("期望 Range 规则"),
        }
    }

    #[test]
    fn test_parse_validator_rules_range_only_max() {
        let mut tags = Tags::new();
        tags.insert("range".to_string(), "..100".to_string());

        let rules = parse_validator_rules(&tags).unwrap();

        match &rules[0] {
            ValidatorRule::Range { min, max } => {
                assert_eq!(min, &None);
                assert_eq!(max, &Some(100));
            }
            _ => panic!("期望 Range 规则"),
        }
    }

    #[test]
    fn test_parse_validator_rules_range_invalid_min_ge_max() {
        let mut tags = Tags::new();
        tags.insert("range".to_string(), "100..1".to_string());

        let result = parse_validator_rules(&tags);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("必须小于"));
    }

    #[test]
    fn test_parse_validator_rules_range_invalid_empty() {
        let mut tags = Tags::new();
        tags.insert("range".to_string(), "..".to_string());

        let result = parse_validator_rules(&tags);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_validator_rules_range_missing() {
        let tags = Tags::new();
        let result = parse_validator_rules(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_validator_rules_int_range_various_formats() {
        let test_cases = vec![
            ("0..100", (Some(0), Some(100))),
            ("-10..10", (Some(-10), Some(10))),
            ("0..", (Some(0), None)),
            ("..100", (None, Some(100))),
        ];

        for (input, expected) in test_cases {
            let mut tags = Tags::new();
            tags.insert("range".to_string(), input.to_string());
            let rules = parse_validator_rules(&tags).unwrap();
            assert_eq!(rules.len(), 1);

            match &rules[0] {
                ValidatorRule::Range { min, max } => {
                    assert_eq!(min, &expected.0);
                    assert_eq!(max, &expected.1);
                }
                _ => panic!("期望 Range 规则"),
            }
        }
    }

    #[test]
    fn test_parse_validator_rules_len_str() {
        // 精确长度
        let mut tags = Tags::new();
        tags.insert("len".to_string(), "5".to_string());
        let rules = parse_validator_rules(&tags).unwrap();
        assert_eq!(rules.len(), 1);

        match &rules[0] {
            ValidatorRule::Length { min, max } => {
                assert_eq!(min, &Some(5));
                assert_eq!(max, &Some(5));
            }
            _ => panic!("期望 Length 规则"),
        }

        // 长度范围
        let mut tags = Tags::new();
        tags.insert("len".to_string(), "1..100".to_string());
        let rules = parse_validator_rules(&tags).unwrap();
        assert_eq!(rules.len(), 1);

        match &rules[0] {
            ValidatorRule::Length { min, max } => {
                assert_eq!(min, &Some(1));
                assert_eq!(max, &Some(100));
            }
            _ => panic!("期望 Length 规则"),
        }
    }

    #[test]
    fn test_parse_validator_rules_no_range_tag() {
        let mut tags = Tags::new();
        tags.insert("label".to_string(), "age".to_string());

        let rules = parse_validator_rules(&tags).unwrap();
        println!("[test_parse_validator_rules_no_range_tag]");
        println!("  -> rules.len() = {}, 期望: 0", rules.len());
        assert_eq!(rules.len(), 0);
    }

    // ===== 错误处理测试 =====

    #[test]
    fn test_parse_empty_string() {
        let result = parse_type("", &empty_enum_map(), &empty_bean_map());
        println!("[test_parse_empty_string] parse_type(\"\") = {:?}", result);
        println!("  -> 期望: Err(...)");
    }

    #[test]
    fn test_parse_unknown_type() {
        let result = parse_type("unknown_type", &empty_enum_map(), &empty_bean_map());
        println!(
            "[test_parse_unknown_type] parse_type(\"unknown_type\") = {:?}",
            result
        );
        println!("  -> 期望: Err(...)");
    }

    #[test]
    fn test_parse_unclosed_generic() {
        let result = parse_type("array<int", &empty_enum_map(), &empty_bean_map());
        println!(
            "[test_parse_unclosed_generic] parse_type(\"array<int\") = {:?}",
            result
        );
        println!("  -> 期望: Err(...)");
    }

    #[test]
    fn test_parse_invalid_tag_format() {
        let result = parse_type("int@", &empty_enum_map(), &empty_bean_map());
        println!(
            "[test_parse_invalid_tag_format] parse_type(\"int@\") = {:?}",
            result
        );
        println!("  -> 期望: Err(...)");
    }

    #[test]
    fn test_parse_duplicate_tag() {
        let result = parse_type(
            "int@key=val1|key=val2",
            &empty_enum_map(),
            &empty_bean_map(),
        );
        println!(
            "[test_parse_duplicate_tag] parse_type(\"int@key=val1|key=val2\") = {:?}",
            result
        );
        println!("  -> 期望: Err(...)");
    }

    #[test]
    fn test_parse_unresolved_dotted_path() {
        let result = parse_type("X.Y.Z.Type", &empty_enum_map(), &empty_bean_map());
        println!(
            "[test_parse_unresolved_dotted_path] parse_type(\"X.Y.Z.Type\") = {:?}",
            result
        );
        if let Err(ref err) = result {
            println!("  -> 错误信息: \"{}\"", err);
            println!("  -> 期望包含: \"既不是已注册的枚举也不是已注册的 Bean\"");
        } else {
            println!("  -> 期望: Err(...) 包含 \"既不是已注册的枚举也不是已注册的 Bean\"");
        }
    }
}
