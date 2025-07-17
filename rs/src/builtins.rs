use crate::{
    calls::{ActionIo, Call, Remoting, RemotingAction},
    errors::{Error, Result},
    prelude::{Decode, Encode, Vec},
};
use gbuiltin_proxy::{ProxyType as GearProxyType, Request as GearProxyRequest};
use gprimitives::ActorId;
use scale_info::TypeInfo;

pub trait BuiltinsRemoting: Remoting {}

#[cfg(feature = "gstd")]
impl BuiltinsRemoting for crate::gstd::calls::GStdRemoting {}

#[cfg(feature = "gclient")]
#[cfg(not(target_arch = "wasm32"))]
impl BuiltinsRemoting for crate::gclient::calls::GClientRemoting {}

// todo [sab] make typeinfo types on gear

// todo [sab] package must provide the address 0x8263cd9fc648e101f1cd8585dc0b193445c3750a63bf64a39cdf58de14826299
pub const PROXY_BUILTIN_ID: ActorId = ActorId::new([
    0x82, 0x63, 0xcd, 0x9f, 0xc6, 0x48, 0xe1, 0x01, 0xf1, 0xcd, 0x85, 0x85, 0xdc, 0x0b, 0x19, 0x34,
    0x45, 0xc3, 0x75, 0x0a, 0x63, 0xbf, 0x64, 0xa3, 0x9c, 0xdf, 0x58, 0xde, 0x14, 0x82, 0x62, 0x99,
]);

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

pub struct ProxyBuiltin<R> {
    remoting: R,
}

impl<R: BuiltinsRemoting + Clone> ProxyBuiltin<R> {
    pub fn new(remoting: R) -> Self {
        Self { remoting }
    }

    pub fn add_proxy(
        &self,
        delegate: ActorId,
        proxy_type: impl Into<ProxyType>,
    ) -> impl Call<Output = Vec<u8>, Args = R::Args> {
        let request = ProxyRequest::AddProxy {
            delegate,
            proxy_type: proxy_type.into(),
        };
        RemotingAction::<_, AddProxy>::new(self.remoting.clone(), request)
    }
}

pub struct AddProxy(());

impl AddProxy {
    pub fn encode_call(delegate: ActorId, proxy_type: impl Into<ProxyType>) -> Vec<u8> {
        let request = ProxyRequest::AddProxy {
            delegate,
            proxy_type: proxy_type.into(),
        };
        <AddProxy as ActionIo>::encode_call(&request)
    }
}

impl ActionIo for AddProxy {
    const ROUTE: &'static [u8] = b"";
    type Params = ProxyRequest;
    type Reply = Vec<u8>;

    fn encode_call(value: &ProxyRequest) -> Vec<u8> {
        if !matches!(value, ProxyRequest::AddProxy { .. }) {
            panic!(
                "internal error: invalid param received. Expected `ProxyRequest::AddProxy`, received: {value:?}"
            );
        }

        value.encode()
    }

    fn decode_reply(payload: impl AsRef<[u8]>) -> Result<Self::Reply> {
        let value = payload.as_ref();
        if !value.is_empty() {
            // todo [sab] change to error type
            panic!("Invalid reply received. Expected empty payload, received `{value:?}`");
        }

        Ok(Default::default())
    }
}
