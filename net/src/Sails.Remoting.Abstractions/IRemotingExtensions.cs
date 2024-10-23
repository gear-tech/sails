using Substrate.Gear.Api.Generated.Model.gprimitives;
using System.Collections.Generic;
using System.Threading.Tasks;
using System.Threading;
using EnsureThat;
using ValueUnit = Substrate.NetApi.Model.Types.Primitive.U128;

namespace Sails.Remoting.Abstractions;

public static class IRemotingExtensions
{
    public static Task<(ActorId ProgramId, byte[] EncodedReply)> ActivateAsync(
        this IRemoting remoting,
        CodeId codeId,
        IReadOnlyCollection<byte> salt,
        IReadOnlyCollection<byte> encodedPayload,
        CancellationToken cancellationToken)
        => EnsureArg.IsNotNull(remoting, nameof(remoting))
            .ActivateAsync(
                codeId,
                salt,
                encodedPayload,
                gasLimit: null,
                ZeroValue,
                cancellationToken);

    public static Task<byte[]> MessageAsync(
        this IRemoting remoting,
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        CancellationToken cancellationToken)
        => EnsureArg.IsNotNull(remoting, nameof(remoting))
            .MessageAsync(
                programId,
                encodedPayload,
                gasLimit: null,
                ZeroValue,
                cancellationToken);

    public static Task<byte[]> QueryAsync(
        this IRemoting remoting,
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        CancellationToken cancellationToken)
        => EnsureArg.IsNotNull(remoting, nameof(remoting))
            .QueryAsync(
                programId,
                encodedPayload,
                gasLimit: null,
                ZeroValue,
                cancellationToken);

    private static readonly ValueUnit ZeroValue = new(0);
}
