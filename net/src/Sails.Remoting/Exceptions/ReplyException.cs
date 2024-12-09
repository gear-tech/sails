using System;
using Substrate.Gear.Api.Generated.Model.gear_core_errors.simple;

namespace Sails.Remoting.Exceptions;

public class ReplyException : SailsException
{
    public ErrorReplyReason Reason { get; } = ErrorReplyReason.Unsupported;

    protected ReplyException()
    {
    }

    protected ReplyException(string message)
        : base(message)
    {
    }

    public ReplyException(string message, ErrorReplyReason reason)
        : base(message)
    {
        this.Reason = reason;
    }

    public ReplyException(string message, ErrorReplyReason reason, Exception innerException)
        : base(message, innerException)
    {
        this.Reason = reason;
    }

    protected ReplyException(string message, Exception innerException)
        : base(message, innerException)
    {
    }
}
