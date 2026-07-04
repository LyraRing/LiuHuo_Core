use super::*;

// ==================== 辅助函数 ====================

/// 快速创建 RawEnumItem
fn item(name: &str, value: &str) -> RawEnumItem {
    RawEnumItem {
        name: name.to_string(),
        value: value.to_string(),
        ..Default::default()
    }
}

/// 快速创建带别名的 RawEnumItem
fn item_with_alias(name: &str, alias: &str, value: &str) -> RawEnumItem {
    RawEnumItem {
        name: name.to_string(),
        alias: Some(alias.to_string()),
        value: value.to_string(),
        ..Default::default()
    }
}

/// 快速创建带标签的 RawEnumItem
fn item_with_tag(name: &str, value: &str, tag_key: &str, tag_val: &str) -> RawEnumItem {
    let mut tags = Tags::new();
    tags.insert(tag_key.to_string(), tag_val.to_string());
    RawEnumItem {
        name: name.to_string(),
        value: value.to_string(),
        tags,
        ..Default::default()
    }
}

// ==================== new / basic ====================

#[test]
fn def_enum_new_basic() {
    let raw = RawEnum {
        name: "Color".to_string(),
        module: "test".to_string(),
        comment: Some("颜色枚举".to_string()),
        items: vec![item("Red", "0"), item("Green", "1"), item("Blue", "2")],
        ..Default::default()
    };
    let def_enum = DefEnum::new(&raw);
    println!("[def_enum_new_basic]");
    println!("  name => expected: \"Color\", actual: {:?}", def_enum.name);
    println!(
        "  module => expected: \"test\", actual: {:?}",
        def_enum.module
    );
    println!(
        "  comment => expected: Some(\"颜色枚举\"), actual: {:?}",
        def_enum.comment
    );
    println!(
        "  items.len() => expected: 3, actual: {:?}",
        def_enum.items.len()
    );
    println!(
        "  full_name() => expected: \"test.Color\", actual: {:?}",
        def_enum.full_name()
    );
}

// ==================== full_name ====================

#[test]
fn def_enum_full_name_empty_module() {
    let raw = RawEnum {
        name: "Status".to_string(),
        module: String::new(),
        ..Default::default()
    };
    let def_enum = DefEnum::new(&raw);
    println!("[def_enum_full_name_empty_module]");
    println!(
        "  full_name() => expected: \"Status\", actual: {:?}",
        def_enum.full_name()
    );
}

#[test]
fn def_enum_full_name_with_module() {
    let raw = RawEnum {
        name: "Status".to_string(),
        module: "core".to_string(),
        ..Default::default()
    };
    let def_enum = DefEnum::new(&raw);
    println!("[def_enum_full_name_with_module]");
    println!(
        "  full_name() => expected: \"core.Status\", actual: {:?}",
        def_enum.full_name()
    );
}

// ==================== compile: 空值自动递增 ====================

#[test]
fn def_enum_compile_auto_increment() {
    let raw = RawEnum {
        name: "Direction".to_string(),
        module: String::new(),
        items: vec![
            item("North", ""),
            item("South", ""),
            item("East", ""),
            item("West", ""),
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_auto_increment]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );

    println!(
        "  items[0].value_int => expected: 0, actual: {:?}",
        def_enum.items[0].value_int
    );
    println!(
        "  items[1].value_int => expected: 1, actual: {:?}",
        def_enum.items[1].value_int
    );
    println!(
        "  items[2].value_int => expected: 2, actual: {:?}",
        def_enum.items[2].value_int
    );
    println!(
        "  items[3].value_int => expected: 3, actual: {:?}",
        def_enum.items[3].value_int
    );
    // value 字符串也被更新
    println!(
        "  items[0].value => expected: \"0\", actual: {:?}",
        def_enum.items[0].value
    );
    println!(
        "  items[1].value => expected: \"1\", actual: {:?}",
        def_enum.items[1].value
    );
    println!(
        "  items[2].value => expected: \"2\", actual: {:?}",
        def_enum.items[2].value
    );
    println!(
        "  items[3].value => expected: \"3\", actual: {:?}",
        def_enum.items[3].value
    );
}

// ==================== compile: 显式整数值 ====================

#[test]
fn def_enum_compile_explicit_values() {
    let raw = RawEnum {
        name: "HttpStatus".to_string(),
        module: "http".to_string(),
        items: vec![
            item("Ok", "200"),
            item("NotFound", "404"),
            item("InternalServerError", "500"),
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_explicit_values]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items[0].value_int => expected: 200, actual: {:?}",
        def_enum.items[0].value_int
    );
    println!(
        "  items[1].value_int => expected: 404, actual: {:?}",
        def_enum.items[1].value_int
    );
    println!(
        "  items[2].value_int => expected: 500, actual: {:?}",
        def_enum.items[2].value_int
    );
}

// ==================== compile: 混合空值和显式值 ====================

#[test]
fn def_enum_compile_mixed_auto_and_explicit() {
    let raw = RawEnum {
        name: "Mixed".to_string(),
        module: String::new(),
        items: vec![
            item("First", ""),   // 自动: 0
            item("Second", "5"), // 显式: 5
            item("Third", ""),   // 自动: 6 (上一个值 +1)
            item("Fourth", "2"), // 显式: 2 (值可以回退)
            item("Fifth", ""),   // 自动: 3
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_mixed_auto_and_explicit]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items[0].value_int => expected: 0, actual: {:?}",
        def_enum.items[0].value_int
    );
    println!(
        "  items[1].value_int => expected: 5, actual: {:?}",
        def_enum.items[1].value_int
    );
    println!(
        "  items[2].value_int => expected: 6, actual: {:?}",
        def_enum.items[2].value_int
    );
    println!(
        "  items[3].value_int => expected: 2, actual: {:?}",
        def_enum.items[3].value_int
    );
    println!(
        "  items[4].value_int => expected: 3, actual: {:?}",
        def_enum.items[4].value_int
    );
}

// ==================== compile: 十六进制、二进制、八进制 ====================

#[test]
fn def_enum_compile_hex_bin_oct_values() {
    let raw = RawEnum {
        name: "RadixEnum".to_string(),
        module: "test".to_string(),
        items: vec![
            item("HexVal", "0xFF"),   // 255
            item("BinVal", "0b1101"), // 13
            item("OctVal", "0o77"),   // 63
            item("DecVal", "100"),    // 100
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_hex_bin_oct_values]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items[0].value_int (0xFF) => expected: 255, actual: {:?}",
        def_enum.items[0].value_int
    );
    println!(
        "  items[1].value_int (0b1101) => expected: 13, actual: {:?}",
        def_enum.items[1].value_int
    );
    println!(
        "  items[2].value_int (0o77) => expected: 63, actual: {:?}",
        def_enum.items[2].value_int
    );
    println!(
        "  items[3].value_int (100) => expected: 100, actual: {:?}",
        def_enum.items[3].value_int
    );
}

// ==================== compile: 负数 ====================

#[test]
fn def_enum_compile_negative_values() {
    let raw = RawEnum {
        name: "NegativeTest".to_string(),
        module: String::new(),
        items: vec![
            item("Negative", "-5"),
            item("Zero", ""),      // 自动: -4
            item("Positive", "3"), // 显式: 3
            item("Another", ""),   // 自动: 4
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_negative_values]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items[0].value_int => expected: -5, actual: {:?}",
        def_enum.items[0].value_int
    );
    println!(
        "  items[1].value_int => expected: -4, actual: {:?}",
        def_enum.items[1].value_int
    );
    println!(
        "  items[2].value_int => expected: 3, actual: {:?}",
        def_enum.items[2].value_int
    );
    println!(
        "  items[3].value_int => expected: 4, actual: {:?}",
        def_enum.items[3].value_int
    );
}

// ==================== compile: 名称冲突 ====================

#[test]
fn def_enum_compile_name_conflict() {
    let raw = RawEnum {
        name: "Duplicate".to_string(),
        module: String::new(),
        items: vec![item("SameName", "1"), item("SameName", "2")],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let result = def_enum.complie();
    println!("[def_enum_compile_name_conflict]");
    println!(
        "  result.is_err() => expected: true, actual: {:?}",
        result.is_err()
    );
    if let Err(ref err) = result {
        println!(
            "  error contains '可能与其他项名称或别名冲突' => expected: true, actual: {:?}",
            err.contains("可能与其他项名称或别名冲突")
        );
        println!("  error message: {:?}", err);
    }
}

// ==================== compile: 名称与别名冲突 ====================

#[test]
fn def_enum_compile_name_alias_conflict() {
    let raw = RawEnum {
        name: "AliasConflict".to_string(),
        module: "test".to_string(),
        items: vec![
            item_with_alias("ItemA", "CommonName", "1"),
            item("CommonName", "2"),
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let result = def_enum.complie();
    println!("[def_enum_compile_name_alias_conflict]");
    println!(
        "  result.is_err() => expected: true, actual: {:?}",
        result.is_err()
    );
    if let Err(ref err) = result {
        println!(
            "  error contains '可能与其他项名称或别名冲突' => expected: true, actual: {:?}",
            err.contains("可能与其他项名称或别名冲突")
        );
        println!("  error message: {:?}", err);
    }
}

// ==================== compile: 两个别名冲突 ====================

#[test]
fn def_enum_compile_two_aliases_conflict() {
    let raw = RawEnum {
        name: "AliasConflict2".to_string(),
        module: String::new(),
        items: vec![
            item_with_alias("ItemA", "DupAlias", "1"),
            item_with_alias("ItemB", "DupAlias", "2"),
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let result = def_enum.complie();
    println!("[def_enum_compile_two_aliases_conflict]");
    println!(
        "  result.is_err() => expected: true, actual: {:?}",
        result.is_err()
    );
    if let Err(ref err) = result {
        println!("  error message: {:?}", err);
    }
}

// ==================== compile: 非 flags 含非整数报错 ====================

#[test]
fn def_enum_compile_non_integer_value_in_non_flags() {
    let raw = RawEnum {
        name: "BadEnum".to_string(),
        module: String::new(),
        items: vec![item("A", "1"), item("B", "not_a_number")],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let result = def_enum.complie();
    println!("[def_enum_compile_non_integer_value_in_non_flags]");
    println!(
        "  result.is_err() => expected: true, actual: {:?}",
        result.is_err()
    );
    if let Err(ref err) = result {
        println!(
            "  error contains '无法解析为整数' => expected: true, actual: {:?}",
            err.contains("无法解析为整数")
        );
        println!("  error message: {:?}", err);
    }
}

// ==================== flags: 基本移位表达式 ====================

#[test]
fn def_enum_compile_flags_basic() {
    let raw = RawEnum {
        name: "Permissions".to_string(),
        module: String::new(),
        is_flags: true,
        items: vec![
            item("None", "0"),
            item("Read", "1 << 0"),
            item("Write", "1 << 1"),
            item("Execute", "1 << 2"),
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_flags_basic]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items[0].value_int (None) => expected: 0, actual: {:?}",
        def_enum.items[0].value_int
    );
    println!(
        "  items[1].value_int (Read) => expected: 1, actual: {:?}",
        def_enum.items[1].value_int
    );
    println!(
        "  items[2].value_int (Write) => expected: 2, actual: {:?}",
        def_enum.items[2].value_int
    );
    println!(
        "  items[3].value_int (Execute) => expected: 4, actual: {:?}",
        def_enum.items[3].value_int
    );
}

// ==================== flags: 显式整数 ====================

#[test]
fn def_enum_compile_flags_with_explicit_int_values() {
    let raw = RawEnum {
        name: "FileMode".to_string(),
        module: "fs".to_string(),
        is_flags: true,
        items: vec![item("Read", "4"), item("Write", "2"), item("Execute", "1")],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_flags_with_explicit_int_values]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items[0].value_int (Read) => expected: 4, actual: {:?}",
        def_enum.items[0].value_int
    );
    println!(
        "  items[1].value_int (Write) => expected: 2, actual: {:?}",
        def_enum.items[1].value_int
    );
    println!(
        "  items[2].value_int (Execute) => expected: 1, actual: {:?}",
        def_enum.items[2].value_int
    );
}

// ==================== flags: 组合表达式 ====================

#[test]
fn def_enum_compile_flags_combined_expression() {
    let raw = RawEnum {
        name: "Access".to_string(),
        module: String::new(),
        is_flags: true,
        items: vec![
            item("Read", "1"),
            item("Write", "2"),
            item("ReadWrite", "Read | Write"),
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_flags_combined_expression]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items[0].value_int (Read) => expected: 1, actual: {:?}",
        def_enum.items[0].value_int
    );
    println!(
        "  items[1].value_int (Write) => expected: 2, actual: {:?}",
        def_enum.items[1].value_int
    );
    // ReadWrite = Read | Write = 1 | 2 = 3
    println!(
        "  items[2].value_int (ReadWrite = Read | Write) => expected: 3, actual: {:?}",
        def_enum.items[2].value_int
    );
}

// ==================== flags: 三元组合 ====================

#[test]
fn def_enum_compile_flags_combined_multi_part() {
    let raw = RawEnum {
        name: "MultiFlag".to_string(),
        module: String::new(),
        is_flags: true,
        items: vec![
            item("A", "1"),
            item("B", "2"),
            item("C", "4"),
            item("All", "A | B | C"),
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_flags_combined_multi_part]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    // All = 1 | 2 | 4 = 7
    println!(
        "  items[3].value_int (All = A | B | C) => expected: 7, actual: {:?}",
        def_enum.items[3].value_int
    );
}

// ==================== flags: 十六进制组合 ====================

#[test]
fn def_enum_compile_flags_with_hex_expression() {
    let raw = RawEnum {
        name: "HexFlags".to_string(),
        module: String::new(),
        is_flags: true,
        items: vec![
            item("FlagA", "0x1"),
            item("FlagB", "0x2"),
            item("FlagAB", "FlagA | FlagB"),
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_flags_with_hex_expression]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items[0].value_int (FlagA) => expected: 0x1, actual: {:?}",
        def_enum.items[0].value_int
    );
    println!(
        "  items[1].value_int (FlagB) => expected: 0x2, actual: {:?}",
        def_enum.items[1].value_int
    );
    println!(
        "  items[2].value_int (FlagAB = FlagA | FlagB) => expected: 0x3, actual: {:?}",
        def_enum.items[2].value_int
    );
}

// ==================== flags: 无法解析 ====================

#[test]
fn def_enum_compile_flags_unresolvable() {
    let raw = RawEnum {
        name: "BadFlags".to_string(),
        module: String::new(),
        is_flags: true,
        items: vec![
            item("A", "1"),
            item("B", "UnknownName"), // 既不是整数，也不是已注册的名称
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let result = def_enum.complie();
    println!("[def_enum_compile_flags_unresolvable]");
    println!(
        "  result.is_err() => expected: true, actual: {:?}",
        result.is_err()
    );
    if let Err(ref err) = result {
        println!(
            "  error contains '无法解析' => expected: true, actual: {:?}",
            err.contains("无法解析")
        );
        println!("  error message: {:?}", err);
    }
}

// ==================== flags: 循环依赖 ====================

#[test]
fn def_enum_compile_flags_circular_unresolvable() {
    let raw = RawEnum {
        name: "CircularFlags".to_string(),
        module: String::new(),
        is_flags: true,
        items: vec![
            item("A", "B"), // A 依赖 B
            item("B", "A"), // B 依赖 A — 循环依赖
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let result = def_enum.complie();
    println!("[def_enum_compile_flags_circular_unresolvable]");
    println!(
        "  result.is_err() => expected: true, actual: {:?}",
        result.is_err()
    );
    if let Err(ref err) = result {
        println!(
            "  error contains '无法解析' => expected: true, actual: {:?}",
            err.contains("无法解析")
        );
        println!("  error message: {:?}", err);
    }
}

// ==================== pre_complie: 不会 panic ====================

#[test]
fn def_enum_pre_compile_does_not_panic() {
    let raw = RawEnum {
        name: "Test".to_string(),
        module: "test".to_string(),
        items: vec![item("A", "1"), item("B", "2")],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    def_enum.pre_complie(); // 目前是空实现，但应该安全调用
    let compile_result = def_enum.complie();
    println!("[def_enum_pre_compile_does_not_panic]");
    println!("  pre_complie() called safely");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
}

// ==================== 编译后内部映射表验证 ====================

#[test]
fn def_enum_compile_verifies_mappings() {
    let raw = RawEnum {
        name: "MappingTest".to_string(),
        module: String::new(),
        items: vec![
            item_with_alias("ItemA", "AliasA", "10"),
            item("ItemB", "20"),
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_verifies_mappings]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );

    // 因为是同模块测试（#[path]），可以直接访问私有字段
    println!(
        "  name_to_value[\"ItemA\"] => expected: Some(&10), actual: {:?}",
        def_enum.name_to_value.get("ItemA")
    );
    println!(
        "  name_to_value[\"ItemB\"] => expected: Some(&20), actual: {:?}",
        def_enum.name_to_value.get("ItemB")
    );
    println!(
        "  value_to_name[&10] => expected: Some(&\"ItemA\"), actual: {:?}",
        def_enum.value_to_name.get(&10)
    );
    println!(
        "  value_to_name[&20] => expected: Some(&\"ItemB\"), actual: {:?}",
        def_enum.value_to_name.get(&20)
    );
    println!(
        "  value_to_alias[&10] => expected: Some(&\"AliasA\"), actual: {:?}",
        def_enum.value_to_alias.get(&10)
    );
    println!(
        "  value_to_alias[&20] => expected: None, actual: {:?}",
        def_enum.value_to_alias.get(&20)
    );
}

// ==================== auto_index ====================

#[test]
fn def_enum_auto_index_preserved() {
    let raw = RawEnum {
        name: "IndexTest".to_string(),
        module: String::new(),
        items: vec![
            item("First", "100"),
            item("Second", "200"),
            item("Third", "300"),
        ],
        ..Default::default()
    };
    let def_enum = DefEnum::new(&raw);
    println!("[def_enum_auto_index_preserved]");
    println!(
        "  items[0].auto_index => expected: 0, actual: {:?}",
        def_enum.items[0].auto_index
    );
    println!(
        "  items[1].auto_index => expected: 1, actual: {:?}",
        def_enum.items[1].auto_index
    );
    println!(
        "  items[2].auto_index => expected: 2, actual: {:?}",
        def_enum.items[2].auto_index
    );
}

// ==================== is_flags / is_unique_item_id ====================

#[test]
fn def_enum_is_flags_flag() {
    let raw = RawEnum {
        name: "FlagsFlagTest".to_string(),
        module: String::new(),
        is_flags: true,
        items: vec![item("A", "1")],
        ..Default::default()
    };
    let def_enum = DefEnum::new(&raw);
    println!("[def_enum_is_flags_flag]");
    println!(
        "  is_flags => expected: true, actual: {:?}",
        def_enum.is_flags
    );
}

#[test]
fn def_enum_is_unique_item_id_flag() {
    let raw = RawEnum {
        name: "UniqueTest".to_string(),
        module: String::new(),
        is_unique_item_id: true,
        items: vec![item("A", "1")],
        ..Default::default()
    };
    let def_enum = DefEnum::new(&raw);
    println!("[def_enum_is_unique_item_id_flag]");
    println!(
        "  is_unique_item_id => expected: true, actual: {:?}",
        def_enum.is_unique_item_id
    );
}

// ==================== groups ====================

#[test]
fn def_enum_groups() {
    let raw = RawEnum {
        name: "GroupedEnum".to_string(),
        module: String::new(),
        groups: vec!["group1".to_string(), "group2".to_string()],
        items: vec![item("A", "1")],
        ..Default::default()
    };
    let def_enum = DefEnum::new(&raw);
    println!("[def_enum_groups]");
    println!(
        "  groups.len() => expected: 2, actual: {:?}",
        def_enum.groups.len()
    );
    println!(
        "  groups[0] => expected: \"group1\", actual: {:?}",
        def_enum.groups[0]
    );
    println!(
        "  groups[1] => expected: \"group2\", actual: {:?}",
        def_enum.groups[1]
    );
}

// ==================== tags ====================

#[test]
fn def_enum_tags() {
    let mut tags = Tags::new();
    tags.insert("category".to_string(), "color".to_string());
    let raw = RawEnum {
        name: "TaggedEnum".to_string(),
        module: String::new(),
        tags,
        ..Default::default()
    };
    let def_enum = DefEnum::new(&raw);
    println!("[def_enum_tags]");
    println!(
        "  tags[\"category\"] => expected: Some(&\"color\"), actual: {:?}",
        def_enum.tags.get("category")
    );
}

// ==================== serialize / deserialize ====================

#[test]
fn def_enum_serialize_deserialize() {
    let raw = RawEnum {
        name: "SerdeTest".to_string(),
        module: "test".to_string(),
        comment: Some("测试序列化".to_string()),
        items: vec![item("A", "1"), item_with_alias("B", "Bee", "2")],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    def_enum.complie().unwrap();

    let json = serde_json::to_string(&def_enum).unwrap();
    println!("[def_enum_serialize_deserialize]");
    println!(
        "  json contains \"SerdeTest\" => expected: true, actual: {:?}",
        json.contains("SerdeTest")
    );
    println!(
        "  json contains \"Bee\" => expected: true, actual: {:?}",
        json.contains("Bee")
    );

    // 反序列化回来
    let deserialized: DefEnum = serde_json::from_str(&json).unwrap();
    println!(
        "  deserialized.name => expected: \"SerdeTest\", actual: {:?}",
        deserialized.name
    );
    println!(
        "  deserialized.items.len() => expected: 2, actual: {:?}",
        deserialized.items.len()
    );
    println!(
        "  deserialized.items[1].alias => expected: Some(\"Bee\"), actual: {:?}",
        deserialized.items[1].alias
    );
    // 注意: name_to_value 等私有字段反序列化为空（未调用 complie），这是预期的行为
}

// ==================== 空枚举 ====================

#[test]
fn def_enum_empty_items() {
    let raw = RawEnum {
        name: "EmptyEnum".to_string(),
        module: String::new(),
        items: vec![],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_empty_items]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items.is_empty() => expected: true, actual: {:?}",
        def_enum.items.is_empty()
    );
}

// ==================== DefEnumItem 方法测试 ====================

#[test]
fn def_enum_item_alias_or_name_with_alias() {
    let raw = RawEnumItem {
        name: "OriginalName".to_string(),
        alias: Some("Shorter".to_string()),
        ..Default::default()
    };
    let item = DefEnumItem::new(&raw);
    println!("[def_enum_item_alias_or_name_with_alias]");
    println!(
        "  alias_or_name() => expected: \"Shorter\", actual: {:?}",
        item.alias_or_name()
    );
}

#[test]
fn def_enum_item_alias_or_name_without_alias() {
    let raw = RawEnumItem {
        name: "OriginalName".to_string(),
        alias: None,
        ..Default::default()
    };
    let item = DefEnumItem::new(&raw);
    println!("[def_enum_item_alias_or_name_without_alias]");
    println!(
        "  alias_or_name() => expected: \"OriginalName\", actual: {:?}",
        item.alias_or_name()
    );
}

#[test]
fn def_enum_item_has_tag() {
    let raw = item_with_tag("TaggedItem", "1", "deprecated", "true");
    let item = DefEnumItem::new(&raw);
    println!("[def_enum_item_has_tag]");
    println!(
        "  has_tag(\"deprecated\") => expected: true, actual: {:?}",
        item.has_tag("deprecated")
    );
    println!(
        "  has_tag(\"nonexistent\") => expected: false, actual: {:?}",
        item.has_tag("nonexistent")
    );
}

#[test]
fn def_enum_item_get_tag() {
    let raw = item_with_tag("TaggedItem", "1", "description", "测试项");
    let item = DefEnumItem::new(&raw);
    println!("[def_enum_item_get_tag]");
    println!(
        "  get_tag(\"description\") => expected: Some(&\"测试项\"), actual: {:?}",
        item.get_tag("description")
    );
    println!(
        "  get_tag(\"nonexistent\") => expected: None, actual: {:?}",
        item.get_tag("nonexistent")
    );
}

// ==================== Default trait ====================

#[test]
fn def_enum_default() {
    let def_enum = DefEnum::default();
    println!("[def_enum_default]");
    println!("  name => expected: \"\", actual: {:?}", def_enum.name);
    println!("  module => expected: \"\", actual: {:?}", def_enum.module);
    println!(
        "  items.is_empty() => expected: true, actual: {:?}",
        def_enum.items.is_empty()
    );
    println!(
        "  is_flags => expected: false, actual: {:?}",
        def_enum.is_flags
    );
    println!(
        "  is_unique_item_id => expected: false, actual: {:?}",
        def_enum.is_unique_item_id
    );
}

#[test]
fn def_enum_item_default() {
    let item = DefEnumItem::default();
    println!("[def_enum_item_default]");
    println!("  name => expected: \"\", actual: {:?}", item.name);
    println!("  value => expected: \"\", actual: {:?}", item.value);
    println!("  value_int => expected: 0, actual: {:?}", item.value_int);
    println!("  alias => expected: None, actual: {:?}", item.alias);
    println!("  comment => expected: None, actual: {:?}", item.comment);
}

// ==================== 边界情况: flags 混合自动递增 ====================

#[test]
fn def_enum_compile_flags_with_auto_increment() {
    let raw = RawEnum {
        name: "FlagsWithAuto".to_string(),
        module: String::new(),
        is_flags: true,
        items: vec![
            item("A", ""),     // 自动 0
            item("B", ""),     // 自动 1
            item("AB", "A|B"), // 组合
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_flags_with_auto_increment]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items[0].value_int (A) => expected: 0, actual: {:?}",
        def_enum.items[0].value_int
    );
    println!(
        "  items[1].value_int (B) => expected: 1, actual: {:?}",
        def_enum.items[1].value_int
    );
    // AB = A|B = 0|1 = 1
    println!(
        "  items[2].value_int (AB = A|B) => expected: 1, actual: {:?}",
        def_enum.items[2].value_int
    );
}

// ==================== 边界情况: flags 空字符串组合部分 ====================

#[test]
fn def_enum_compile_flags_empty_parts() {
    let raw = RawEnum {
        name: "FlagsEmptyParts".to_string(),
        module: String::new(),
        is_flags: true,
        items: vec![
            item("A", "1"),
            item("B", "2"),
            item("Combined", "A | "), // 尾部有空白但无值
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_flags_empty_parts]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    // A | "" → filter 后只剩 A
    println!(
        "  items[2].value_int (Combined = A | ) => expected: 1, actual: {:?}",
        def_enum.items[2].value_int
    );
}

// ==================== 边界情况: flags 中的 shift 表达式 ====================

#[test]
fn def_enum_compile_flags_with_shift_in_expression() {
    let raw = RawEnum {
        name: "ShiftFlags".to_string(),
        module: String::new(),
        is_flags: true,
        items: vec![
            item("FlagA", "1 << 0"),                 // 1
            item("FlagB", "1 << 1"),                 // 2
            item("FlagC", "1 << 2"),                 // 4
            item("All", "1 << 0 | 1 << 1 | 1 << 2"), // 7
        ],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    let compile_result = def_enum.complie();
    println!("[def_enum_compile_flags_with_shift_in_expression]");
    println!(
        "  compile() => expected: Ok(()), actual: {:?}",
        compile_result
    );
    println!(
        "  items[3].value_int (All) => expected: 7, actual: {:?}",
        def_enum.items[3].value_int
    );
}

// ==================== Debug trait ====================

#[test]
fn def_enum_debug_format() {
    let raw = RawEnum {
        name: "DebugEnum".to_string(),
        module: String::new(),
        items: vec![item("X", "42")],
        ..Default::default()
    };
    let mut def_enum = DefEnum::new(&raw);
    def_enum.complie().unwrap();
    let debug_str = format!("{:?}", def_enum);
    println!("[def_enum_debug_format]");
    println!(
        "  debug_str contains \"DebugEnum\" => expected: true, actual: {:?}",
        debug_str.contains("DebugEnum")
    );
    println!(
        "  debug_str contains \"42\" => expected: true, actual: {:?}",
        debug_str.contains("42")
    );
    println!("  debug_str: {:?}", debug_str);
}
