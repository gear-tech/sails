use anyhow::{Result, anyhow};
use sails_idl_parser::ast::{PrimitiveType, TypeDecl, TypeId};

pub trait TypeDeclToSol {
    fn get_ty(&self) -> Result<String>;
    fn get_mem_location(&self) -> Option<String>;
}

impl TypeDeclToSol for TypeDecl {
    fn get_ty(&self) -> Result<String> {
        match self {
            TypeDecl::Id(ty) => ty.get_ty(),
            TypeDecl::Array { item, len } => Ok(format!("{}[{}]", item.get_ty()?, len)),
            TypeDecl::Vector(item) => Ok(format!("{}[]", item.get_ty()?)),
            _ => Err(anyhow!("type is not supported")),
        }
    }

    fn get_mem_location(&self) -> Option<String> {
        match self {
            TypeDecl::Id(ty) => ty.get_mem_location(),
            TypeDecl::Array { item: _, len: _ } => Some("memory".to_string()),
            TypeDecl::Vector(_) => Some("memory".to_string()),
            _ => None,
        }
    }
}

impl TypeDeclToSol for PrimitiveType {
    fn get_ty(&self) -> Result<String> {
        Ok(match self {
            Self::Bool => "bool",
            Self::U8 => "uint8",
            Self::U16 => "uint16",
            Self::U32 => "uint32",
            Self::U64 => "uint64",
            Self::U128 => "uint128",
            Self::U256 => "uint256",
            Self::I8 => "int8",
            Self::I16 => "int16",
            Self::I32 => "int32",
            Self::I64 => "int64",
            Self::I128 => "int128",
            Self::Str => "string",
            Self::ActorId | Self::H256 | Self::CodeId | Self::MessageId => "bytes32",
            Self::H160 => "bytes20",
            _ => return Err(anyhow!("type is not supported")),
        }
        .to_string())
    }

    fn get_mem_location(&self) -> Option<String> {
        match self {
            Self::Str => Some("memory".to_string()),
            _ => None,
        }
    }
}

impl TypeDeclToSol for TypeId {
    fn get_ty(&self) -> Result<String> {
        match self {
            Self::Primitive(ty) => ty.get_ty(),
            _ => Err(anyhow!("type is not supported")),
        }
    }

    fn get_mem_location(&self) -> Option<String> {
        match self {
            Self::Primitive(ty) => ty.get_mem_location(),
            _ => None,
        }
    }
}
