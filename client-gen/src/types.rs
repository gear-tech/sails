use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idl {
    pub types: Vec<BaseType>,
    pub services: Vec<Service>,
}

/// Top level type, which also contains how the type is named
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]

pub struct BaseType {
    #[serde(rename = "type")]
    pub tt: SimpleTypeDecl,
    #[serde(flatten)]
    pub def: InnerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum InnerType {
    #[serde(rename = "typeName")]
    TypeName { def: SimpleTypeDecl },
    #[serde(rename = "option")]
    Option { def: Box<InnerType> },
    #[serde(rename = "result")]
    Result { def: ResultDef },
    #[serde(rename = "vec")]
    Vec { def: Box<InnerType> },
    #[serde(rename = "tuple")]
    Tuple { def: TupleDef },
    #[serde(rename = "struct")]
    Struct { def: StructDef },
    #[serde(rename = "variant")]
    Enum { def: EnumDef },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleTypeDecl {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultDef {
    pub ok: Box<InnerType>,
    pub err: Box<InnerType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TupleDef {
    pub fields: Vec<InnerType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDef {
    pub fields: Vec<StructFieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructFieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub tt: InnerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDef {
    pub variants: Vec<VariantFieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantFieldDef {
    pub name: String,
    #[serde(rename = "type", default)]
    pub tt: Option<InnerType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub methods: Vec<ServiceMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMethodKind {
    #[serde(rename = "message")]
    Command,
    #[serde(rename = "query")]
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMethod {
    pub def: ServiceMethodDef,
    pub kind: ServiceMethodKind,
}

impl ServiceMethod {
    pub fn args_struct(&self) -> BaseType {
        BaseType {
            tt: SimpleTypeDecl {
                name: format!("{}RequestArgs", self.def.name.to_case(Case::Pascal)),
            },
            def: InnerType::Struct {
                def: StructDef {
                    fields: self
                        .def
                        .args
                        .iter()
                        .map(|arg| StructFieldDef {
                            name: arg.name.clone(),
                            tt: arg.tt.clone(),
                        })
                        .collect::<Vec<_>>(),
                },
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMethodDef {
    pub name: String,
    pub args: Vec<ServiceMethodArg>,
    pub output: Box<InnerType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMethodArg {
    pub name: String,
    #[serde(rename = "type")]
    pub tt: InnerType,
}

#[cfg(test)]
mod tests {
    use crate::{BaseType, InnerType};
    use serde_json::json;

    #[test]
    fn test_simple() {
        let v = json!({
            "def": {
                "name": "u32",
                "kind": "simple"
            },
            "kind": "typeName",
            "type": {
                "name": "Alias",
                "kind": "simple"
            }
        });

        let r: BaseType = serde_json::from_value(v).unwrap();

        assert!(matches!(r.def, InnerType::TypeName { .. }))
    }

    #[test]
    fn test_struct() {
        let v = json!({
            "def": {
                "fields": [
                    {
                        "name": "p1",
                        "type": {
                            "def": {
                                "name": "u32",
                                "kind": "simple"
                            },
                            "kind": "typeName"
                        }
                    },
                    {
                        "name": "p2",
                        "type": {
                            "def": {
                                "name": "string",
                                "kind": "simple"
                            },
                            "kind": "typeName"
                        }
                    }
                ]
            },
            "kind": "struct",
            "type": {
                "name": "ThisThatAppDoThatParam",
                "kind": "simple"
            }
        });

        let r = parse(v);

        assert!(matches!(r.def, InnerType::Struct { .. }))
    }

    #[test]
    fn test_vec() {
        let v = json!({
            "def": {
                "def": {
                    "name": "u8",
                    "kind": "simple"
                },
                "kind": "typeName"
            },
            "kind": "vec",
            "type": {
                "name": "VecAlias",
                "kind": "simple"
            }
        });

        let r = parse(v);

        assert!(matches!(r.def, InnerType::Vec { .. }))
    }

    fn parse(v: serde_json::Value) -> BaseType {
        let s = v.to_string();
        let jd = &mut serde_json::Deserializer::from_str(&s);

        match serde_path_to_error::deserialize(jd) {
            Ok(r) => r,
            Err(err) => {
                panic!("{}: {}", err.path(), err)
            }
        }
    }
}
