use crate::{
    calls::{ActionIo, Call, Remoting, RemotingAction},
    errors::{Error, Result},
    prelude::{Decode, Encode, Vec},
};
use gbuiltin_proxy::{ProxyType, Request as ProxyRequest};
use gprimitives::ActorId;

// 0x8263cd9fc648e101f1cd8585dc0b193445c3750a63bf64a39cdf58de14826299
const PROXY_BUILTIN_ID: ActorId = ActorId::new([
    0x82, 0x63, 0xcd, 0x9f, 0xc6, 0x48, 0xe1, 0x01, 0xf1, 0xcd, 0x85, 0x85, 0xdc, 0x0b, 0x19, 0x34,
    0x45, 0xc3, 0x75, 0x0a, 0x63, 0xbf, 0x64, 0xa3, 0x9c, 0xdf, 0x58, 0xde, 0x14, 0x82, 0x62, 0x99,
]);

pub struct ProxyBuiltin<R> {
    remoting: R,
}

impl<R: Remoting + Clone> ProxyBuiltin<R> {
    pub fn new(remoting: R) -> Self {
        Self { remoting }
    }

    pub fn add_proxy(
        &self,
        delegate: ActorId,
        proxy_type: ProxyType,
    ) -> impl Call<Output = Vec<u8>, Args = R::Args> {
        let request = ProxyRequest::AddProxy {
            delegate,
            proxy_type,
        };
        RemotingAction::<_, AddProxy>::new(self.remoting.clone(), request)
    }
}

pub struct AddProxy(());

impl AddProxy {
    pub fn encode_call(delegate: ActorId, proxy_type: ProxyType) -> Vec<u8> {
        let request = ProxyRequest::AddProxy {
            delegate,
            proxy_type,
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
                "Invalid param received. Expected `ProxyRequest::AddProxy`, received `{value:?}`"
            );
        }

        value.encode()
    }

    fn decode_reply(payload: impl AsRef<[u8]>) -> Result<Self::Reply> {
        let mut value = payload.as_ref();

        Decode::decode(&mut value).map_err(Error::Codec)
    }
}
