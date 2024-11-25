using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Substrate.Gear.Api.Generated.Model.gprimitives;

namespace Sails.Remoting.Abstractions.Core;

public static class RemotingExtensions
{
    /// <inheritdoc cref="IRemoting.ActivateAsync(CodeId, IReadOnlyCollection{byte}, IReadOnlyCollection{byte}, GasUnit?, ValueUnit, CancellationToken)"/>
    public static Task<RemotingReply<(ActorId ProgramId, byte[] Payload)>> ActivateAsync(
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

    /// <inheritdoc cref="IRemoting.MessageAsync(ActorId, IReadOnlyCollection{byte}, GasUnit?, ValueUnit, CancellationToken)"/>
    public static Task<RemotingReply<byte[]>> MessageAsync(
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

    /// <inheritdoc cref="IRemoting.QueryAsync(ActorId, IReadOnlyCollection{byte}, GasUnit?, ValueUnit, CancellationToken)"/>
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
