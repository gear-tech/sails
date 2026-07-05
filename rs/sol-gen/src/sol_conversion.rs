use alloc::string::String;
use sails_idl_parser_v2::ast::{PrimitiveType, TypeDecl};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Type is not supported")]
    UnsupportedType,
}

pub trait TypeDeclExt {
    fn get_ty(&self) -> Result<String, ConversionError>;

    fn get_mem_location(&self) -> Option<String>;
}

impl TypeDeclExt for TypeDecl {
    fn get_ty(&self) -> Result<String, ConversionError> {
        match self {
            TypeDecl::Primitive(ty) => ty.get_ty(),
            _ => Err(ConversionError::UnsupportedType),
        }
    }

    fn get_mem_location(&self) -> Option<String> {
        match self {
            TypeDecl::Primitive(ty) => ty.get_mem_location(),
            TypeDecl::Array { .. } => Some("calldata".into()),
            TypeDecl::Slice { .. } => Some("calldata".into()),
            _ => None,
        }
    }
}

impl TypeDeclExt for PrimitiveType {
    fn get_ty(&self) -> Result<String, ConversionError> {
        Ok(match self {
            Self::Bool => "bool".into(),
            Self::U8 => "uint8".into(),
            Self::U16 => "uint16".into(),
            Self::U32 => "uint32".into(),
            Self::U64 => "uint64".into(),
            Self::U128 => "uint128".into(),
            Self::U256 => "uint256".into(),
            Self::I8 => "int8".into(),
            Self::I16 => "int16".into(),
            Self::I32 => "int32".into(),
            Self::I64 => "int64".into(),
            Self::I128 => "int128".into(),
            Self::String => "string".into(),
            Self::ActorId => "address".into(),
            Self::H256 | Self::CodeId | Self::MessageId => "bytes32".into(),
            Self::H160 => "bytes20".into(),
            _ => return Err(ConversionError::UnsupportedType),
        })
    }

    fn get_mem_location(&self) -> Option<String> {
        match self {
            Self::String => Some("calldata".into()),
            _ => None,
        }
    }
}
