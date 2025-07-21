use crate::{
    ActorId,
    builtins::{BuiltinsRemoting, builtin_action},
    calls::{ActionIo, Call, RemotingAction},
    errors::Result,
    prelude::{Decode, Encode, TypeInfo, Vec},
};
use gbuiltin_proxy::{ProxyType as GearProxyType, Request as GearProxyRequest};

// todo [sab] make typeinfo types on gear
// todo [sab] package must provide the address

/// Gear protocol proxy builtin id is 0x8263cd9fc648e101f1cd8585dc0b193445c3750a63bf64a39cdf58de14826299
pub const PROXY_BUILTIN_ID: ActorId = ActorId::new([
    0x82, 0x63, 0xcd, 0x9f, 0xc6, 0x48, 0xe1, 0x01, 0xf1, 0xcd, 0x85, 0x85, 0xdc, 0x0b, 0x19, 0x34,
    0x45, 0xc3, 0x75, 0x0a, 0x63, 0xbf, 0x64, 0xa3, 0x9c, 0xdf, 0x58, 0xde, 0x14, 0x82, 0x62, 0x99,
]);

builtin_action! {
    ProxyRequest,
    ProxyBuiltin,
    AddProxy { delegate: ActorId, proxy_type: ProxyType }
}

builtin_action! {
    ProxyRequest,
    ProxyBuiltin,
    RemoveProxy { delegate: ActorId, proxy_type: ProxyType }
}

pub struct ProxyBuiltin<R> {
    remoting: R,
}

impl<R: BuiltinsRemoting + Clone> ProxyBuiltin<R> {
    pub fn new(remoting: R) -> Self {
        Self { remoting }
    }
}

/// `TypeInfo` implementor copy of `gbuiltin_proxy::Request`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum ProxyRequest {
    AddProxy {
        delegate: ActorId,
        proxy_type: ProxyType,
    },
    RemoveProxy {
        delegate: ActorId,
        proxy_type: ProxyType,
    },
}

impl From<GearProxyRequest> for ProxyRequest {
    fn from(request: GearProxyRequest) -> Self {
        match request {
            GearProxyRequest::AddProxy {
                delegate,
                proxy_type,
            } => Self::AddProxy {
                delegate,
                proxy_type: proxy_type.into(),
            },
            GearProxyRequest::RemoveProxy {
                delegate,
                proxy_type,
            } => Self::RemoveProxy {
                delegate,
                proxy_type: proxy_type.into(),
            },
        }
    }
}

/// `TypeInfo` implementor copy of `gbuiltin_proxy::ProxyType`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum ProxyType {
    Any,
    NonTransfer,
    Governance,
    Staking,
    IdentityJudgement,
    CancelProxy,
}

impl From<GearProxyType> for ProxyType {
    fn from(proxy_type: GearProxyType) -> Self {
        match proxy_type {
            GearProxyType::Any => Self::Any,
            GearProxyType::NonTransfer => Self::NonTransfer,
            GearProxyType::Governance => Self::Governance,
            GearProxyType::Staking => Self::Staking,
            GearProxyType::IdentityJudgement => Self::IdentityJudgement,
            GearProxyType::CancelProxy => Self::CancelProxy,
        }
    }
}

#[test]
fn test_id() {
    let expected = hex::decode("8263cd9fc648e101f1cd8585dc0b193445c3750a63bf64a39cdf58de14826299")
        .expect("Failed to decode hex");
    assert_eq!(PROXY_BUILTIN_ID.into_bytes().to_vec(), expected);
}
