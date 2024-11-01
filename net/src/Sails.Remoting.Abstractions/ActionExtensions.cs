using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting.Abstractions;

public static class ActionExtensions
{
    /// <summary>
    /// Activates/creates a program from previously uploaded code and receive ProgramId
    /// </summary>
    /// <param name="activation"></param>
    /// <param name="codeId">Code identifier. This identifier can be obtained as a result of executing the gear.uploadCode extrinsic.</param>
    /// <param name="salt">Salt bytes</param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static async Task<ActorId> SendReceiveAsync(
        this IActivation activation,
        CodeId codeId,
        IReadOnlyCollection<byte> salt,
        CancellationToken cancellationToken)
    {
        await using var reply = await activation.ActivateAsync(codeId, salt, cancellationToken).ConfigureAwait(false);
        return await reply.ReceiveAsync(cancellationToken).ConfigureAwait(false);
    }


    /// <summary>
    /// Sends a message to a program for execution and receive reply
    /// </summary>
    /// <typeparam name="T"></typeparam>
    /// <param name="call"></param>
    /// <param name="programId">Program identifier</param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public static async Task<T> SendReceiveAsync<T>(
        this ICall<T> call,
        ActorId programId,
        CancellationToken cancellationToken)
        where T : IType, new()
    {
        await using var reply = await call.MessageAsync(programId, cancellationToken).ConfigureAwait(false);
        return await reply.ReceiveAsync(cancellationToken).ConfigureAwait(false);
    }
}
