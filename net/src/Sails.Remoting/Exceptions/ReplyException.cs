using System;
using Substrate.Gear.Api.Generated.Model.gear_core_errors.simple;

namespace Sails.Remoting.Exceptions;

public class ReplyException : Exception
{
    public ErrorReplyReason Reason { get; }

    public SimpleExecutionError? ExecutionError { get; }

    public SimpleProgramCreationError? ProgramCreationError { get; }

    public ReplyException()
    {
    }

    public ReplyException(string message) : base(message)
    {
    }

    public ReplyException(string message, ErrorReplyReason reason) : base(message)
    {
        this.Reason = reason;
    }

    public ReplyException(
        string message,
        ErrorReplyReason reason,
        SimpleExecutionError? executionError,
        SimpleProgramCreationError? programCreationError) : base(message)
    {
        this.Reason = reason;
        this.ExecutionError = executionError;
        this.ProgramCreationError = programCreationError;
    }


    public ReplyException(
        string message,
        ErrorReplyReason reason,
        SimpleExecutionError? executionError,
        SimpleProgramCreationError? programCreationError,
        Exception innerException) : base(message, innerException)
    {
        this.Reason = reason;
        this.ExecutionError = executionError;
        this.ProgramCreationError = programCreationError;
    }

    public ReplyException(string message, Exception innerException) : base(message, innerException)
    {
    }
}
