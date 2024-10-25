using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Types;
using GasUnit = Substrate.NetApi.Model.Types.Primitive.U64;
using ValueUnit = Substrate.NetApi.Model.Types.Primitive.U128;

namespace Sails.Remoting.Abstractions;

public interface IRemoting
{
    /// <summary>
    /// Sets account for signing transactions.
    /// </summary>
    /// <param name="signingAccount"></param>
    void SetSigningAccount(Account signingAccount);

    /// <summary>
    /// Activates/creates a program from previously uploaded code.
    /// </summary>
    /// <param name="codeId"></param>
    /// <param name="salt"></param>
    /// <param name="encodedPayload"></param>
    /// <param name="gasLimit"></param>
    /// <param name="value"></param>
    /// <param name="cancellationToken"></param>
    /// <returns>A task for obtaining activated program ID and SCALE-encoded reply.</returns>
    Task<Task<(ActorId ProgramId, byte[] EncodedReply)>> ActivateAsync(
        CodeId codeId,
        IReadOnlyCollection<byte> salt,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit? gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken);

    /// <summary>
    /// Sends a message to a program for execution.
    /// </summary>
    /// <param name="programId"></param>
    /// <param name="encodedPayload"></param>
    /// <param name="gasLimit"></param>
    /// <param name="value"></param>
    /// <param name="cancellationToken"></param>
    /// <returns>A task for obtaining SCALE-encoded reply.</returns>
    Task<Task<byte[]>> MessageAsync(
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit? gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken);

    /// <summary>
    /// Queries a program for information.
    /// </summary>
    /// <param name="programId"></param>
    /// <param name="encodedPayload"></param>
    /// <param name="gasLimit"></param>
    /// <param name="value"></param>
    /// <param name="cancellationToken"></param>
    /// <returns>SCALE-encoded reply.</returns>
    Task<byte[]> QueryAsync(
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit? gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken);
}
