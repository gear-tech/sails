using Substrate.Gear.Api.Generated.Model.gear_core_errors.simple;

namespace Substrate.Gear.Client;

public sealed record ReplyInfo
{
    /// <summary>
    /// Payload of the reply.
    /// </summary>
    public required byte[] EncodedPayload { get; init; }
    /// <summary>
    /// Value sent with the reply.
    /// /// </summary>
    public required ValueUnit Value { get; init; }
    /// <summary>
    /// Reply code of the reply.
    /// <summary>
    public required EnumReplyCode Code { get; init; }
}
