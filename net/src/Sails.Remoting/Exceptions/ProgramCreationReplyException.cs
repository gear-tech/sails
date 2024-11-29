using System;
using Substrate.Gear.Api.Generated.Model.gear_core_errors.simple;

namespace Sails.Remoting.Exceptions;

public class ProgramCreationReplyException : ReplyException
{
    public SimpleProgramCreationError ProgramCreationError { get; }

    protected ProgramCreationReplyException()
    {
    }

    protected ProgramCreationReplyException(string message) : base(message)
    {
    }

    public ProgramCreationReplyException(string message, ErrorReplyReason reason, SimpleProgramCreationError programCreationError)
        : base(message, reason)
    {
        this.ProgramCreationError = programCreationError;
    }

    public ProgramCreationReplyException(
        string message,
        ErrorReplyReason reason,
        SimpleProgramCreationError programCreationError,
        Exception innerException)
    : base(message, reason, innerException)
    {
        this.ProgramCreationError = programCreationError;
    }


    protected ProgramCreationReplyException(string message, ErrorReplyReason reason) : base(message, reason)
    {
    }

    protected ProgramCreationReplyException(string message, ErrorReplyReason reason, Exception innerException)
        : base(message, reason, innerException)
    {
    }

    protected ProgramCreationReplyException(string message, Exception innerException)
        : base(message, innerException)
    {
    }
}
