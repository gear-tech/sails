using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Substrate.Gear.Api.Generated.Model.gprimitives;

namespace Sails.Remoting.Abstractions;

public interface IActivation : IActionBuilder<IActivation>
{
    /// <summary>
    /// Activates/creates a program from previously uploaded code
    /// </summary>
    /// <param name="codeId">Code identifier. This identifier can be obtained as a result of executing the gear.uploadCode extrinsic.</param>
    /// <param name="salt">Salt bytes</param>
    /// <param name="cancellationToken">Propagates notification that operations should be canceled. <see cref="CancellationToken"/> </param>
    /// <returns>Reply with Program identifier. <see cref="IReply{T}"/></returns>
    Task<IReply<ActorId>> ActivateAsync(
        CodeId codeId,
        IReadOnlyCollection<byte> salt,
        CancellationToken cancellationToken);
}
