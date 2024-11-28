﻿using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Substrate.Gear.Api.Generated.Model.gprimitives;

namespace Sails.Remoting.Abstractions.Core;

public interface IRemoting
{
    /// <summary>
    /// Activates/creates a program from previously uploaded code.
    /// </summary>
    /// <param name="codeId"></param>
    /// <param name="salt"></param>
    /// <param name="encodedPayload"></param>
    /// <param name="gasLimit"></param>
    /// <param name="value"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    Task<RemotingReply<(ActorId ProgramId, byte[] Payload)>> ActivateAsync(
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
    /// <returns></returns>
    Task<RemotingReply<byte[]>> MessageAsync(
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
    /// <returns></returns>
    Task<byte[]> QueryAsync(
        ActorId programId,
        IReadOnlyCollection<byte> encodedPayload,
        GasUnit? gasLimit,
        ValueUnit value,
        CancellationToken cancellationToken);

    /// <summary>
    /// Asynchronously subscribe to Gear events.
    /// </summary>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    Task<EventListener<(ActorId Source, byte[] Payload)>> ListenAsync(CancellationToken cancellationToken);
}
