using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using GasUnit = Substrate.NetApi.Model.Types.Primitive.U64;
using ValueUnit = Substrate.NetApi.Model.Types.Primitive.U128;

namespace Sails.Remoting.Abstractions;

public static class IRemotingExtensions
{
    /// <inheritdoc cref="IRemoting.ActivateAsync(CodeId, IReadOnlyCollection{byte}, IReadOnlyCollection{byte}, GasUnit?, ValueUnit, CancellationToken)"/>
    public static Task<ActivationResult> ActivateAsync(
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
    public static Task<Task<byte[]>> MessageAsync(
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
