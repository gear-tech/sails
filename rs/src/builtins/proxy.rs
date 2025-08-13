use crate::{
    ActorId,
    builtins::{BuiltinsRemoting, builtin_action},
    calls::{ActionIo, Call, RemotingAction},
    errors::Result,
    prelude::{Decode, Encode, TypeInfo, Vec},
};
use gbuiltin_proxy::{ProxyType as GearProxyType, Request as GearProxyRequest};

// todo [sab] make typeinfo types on gear

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

pub trait ProxyBuiltinTrait {
    type Args;

    /// Adds a proxy for the specified delegate with the given proxy type.
    fn add_proxy(
        &self,
        delegate: ActorId,
        proxy_type: ProxyType,
    ) -> impl Call<Output = (), Args = Self::Args>;

    /// Removes a proxy for the specified delegate with the given proxy type.
    fn remove_proxy(
        &self,
        delegate: ActorId,
        proxy_type: ProxyType,
    ) -> impl Call<Output = (), Args = Self::Args>;
}

impl<R: BuiltinsRemoting + Clone> ProxyBuiltinTrait for ProxyBuiltin<R> {
    type Args = R::Args;

    fn add_proxy(
        &self,
        delegate: ActorId,
        proxy_type: ProxyType,
    ) -> impl Call<Output = (), Args = Self::Args> {
        self.add_proxy(delegate, proxy_type)
    }

    fn remove_proxy(
        &self,
        delegate: ActorId,
        proxy_type: ProxyType,
    ) -> impl Call<Output = (), Args = Self::Args> {
        self.remove_proxy(delegate, proxy_type)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::test_utils::assert_action_codec;

    #[test]
    fn test_codec() {
        assert_action_codec!(
            ProxyRequest,
            AddProxy {
                delegate: ActorId::from([1; 32]),
                proxy_type: ProxyType::Any
            }
        );
        assert_action_codec!(
            ProxyRequest,
            RemoveProxy {
                delegate: ActorId::from([2; 32]),
                proxy_type: ProxyType::NonTransfer
            }
        );
    }
}
