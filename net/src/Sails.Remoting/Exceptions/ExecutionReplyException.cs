using System;
using Substrate.Gear.Api.Generated.Model.gear_core_errors.simple;

namespace Sails.Remoting.Exceptions;

public class ExecutionReplyException : ReplyException
{
    public SimpleExecutionError ExecutionError { get; } = SimpleExecutionError.Unsupported;

    protected ExecutionReplyException()
    {
    }

    protected ExecutionReplyException(string message)
        : base(message)
    {
    }

    public ExecutionReplyException(string message, ErrorReplyReason reason, SimpleExecutionError executionError)
        : base(message, reason)
    {
        this.ExecutionError = executionError;
    }

    public ExecutionReplyException(
        string message,
        ErrorReplyReason reason,
        SimpleExecutionError executionError,
        Exception innerException)
        : base(message, reason, innerException)
    {
        this.ExecutionError = executionError;
    }

    protected ExecutionReplyException(string message, ErrorReplyReason reason)
        : base(message, reason)
    {
    }

    protected ExecutionReplyException(string message, ErrorReplyReason reason, Exception innerException)
        : base(message, reason, innerException)
    {
    }

    protected ExecutionReplyException(string message, Exception innerException)
        : base(message, innerException)
    {
    }
}
