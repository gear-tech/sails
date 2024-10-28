using EnsureThat;
using Substrate.NetApi;
using Substrate.NetApi.Model.Extrinsics;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.Model.Extrinsics;

public static class ExtrinsicExtensions
{
    /// <summary>
    /// Encodes specified extrinsic using SCALE codec, and then calculates Blake2 hash
    /// of the encoded bytes for submission to blockchain.
    /// </summary>
    /// <param name="extrinsic"></param>
    /// <returns></returns>
    public static (byte[] EncodedBytes, Hash Hash) EncodeAndHash(this Extrinsic extrinsic)
    {
        EnsureArg.IsNotNull(extrinsic, nameof(extrinsic));

        var encodedBytes = extrinsic.Encode();
        var hash = new Hash(HashExtension.Blake2(encodedBytes, 256));
        return (encodedBytes, hash);
    }
}
