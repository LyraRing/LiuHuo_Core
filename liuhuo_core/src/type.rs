use crate::{
    Tags,
    defs::{BeanId, DefAssembly, EnumId},
    utility::parse_type,
};

pub const TAG_KEY_RANGE: &str = "range";

pub type Vec2I = (usize, usize);

/// 定义类型的元信息，包括是否可空和验证规则。
#[derive(Debug)]
pub struct TypeMeta {
    pub nullable: bool,
    pub tags: Tags,
    pub validator_rules: Vec<ValidatorRule>,
}

/// 定义类型的枚举，包括基础类型、枚举类型、Bean 类型和集合类型。
#[derive(Debug)]
pub enum TypeKind {
    Bool(TypeMeta),   // 布尔类型
    I32(TypeMeta),    // 32 位整数类型
    I64(TypeMeta),    // 64 位整数类型
    F32(TypeMeta),    // 32 位浮点数类型
    F64(TypeMeta),    // 64 位浮点数类型
    String(TypeMeta), // 字符串类型

    Enum {
        meta: TypeMeta,
        def: EnumId,
    }, // 枚举类型
    Bean {
        meta: TypeMeta,
        def: BeanId,
    }, // Bean 类型

    Array {
        elem_type: Box<TypeKind>,
        dimensions: usize,
    }, // 数组类型
    List {
        elem_type: Box<TypeKind>,
    }, // 列表类型
    Set {
        elem_type: Box<TypeKind>,
    }, // 集合类型
    Map {
        key_type: Box<TypeKind>,
        value_type: Box<TypeKind>,
    }, // 映射类型
    Matrix {
        elem_type: Box<TypeKind>,
        dimensions: Vec2I,
    }, // 矩阵类型
}

/// 定义了键类型的枚举。
pub enum KeyType {
    Bool,
    I32,
    I64,
    String,
    Enum(EnumId),
}

#[derive(Debug, Clone)]
pub enum ValidatorRule {
    Range {
        min: Option<i64>,
        max: Option<i64>,
    }, // 范围验证规则，包含最小值和最大值
    RangeF {
        min: Option<f64>,
        max: Option<f64>,
    }, // 范围验证规则，包含最小值和最大值（浮点数）
    Length {
        min: Option<usize>,
        max: Option<usize>,
    }, // 长度验证规则，包含最小长度和最大长度
    Regex(String), // 正则表达式验证规则
}

impl TypeKind {
    /// 检查类型是否为基础类型
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            TypeKind::Bool(_)
                | TypeKind::I32(_)
                | TypeKind::I64(_)
                | TypeKind::F32(_)
                | TypeKind::F64(_)
                | TypeKind::String(_)
        )
    }

    /// 检查类型是否为枚举类型
    pub fn is_enum(&self) -> bool {
        matches!(self, TypeKind::Enum { .. })
    }

    /// 检查类型是否为 Bean 类型
    pub fn is_bean(&self) -> bool {
        matches!(self, TypeKind::Bean { .. })
    }

    /// 检查类型是否为集合类型（数组、列表、集合、映射）
    pub fn is_collection(&self) -> bool {
        matches!(
            self,
            TypeKind::Array { .. }
                | TypeKind::List { .. }
                | TypeKind::Set { .. }
                | TypeKind::Map { .. }
        )
    }

    pub fn parse(type_str: &str, asm: &DefAssembly) -> Result<Self, String> {
        // 解析类型字符串，返回对应的 TypeKind 实例
        // 这里需要实现具体的解析逻辑，根据 type_str 的格式来判断是基础类型、枚举类型、Bean 类型还是集合类型
        // 如果解析失败，返回 Err(String) 错误信息
        parse_type(
            type_str,
            &asm.full_name_to_enum_id,
            &asm.full_name_to_bean_id,
        )
    }
}
