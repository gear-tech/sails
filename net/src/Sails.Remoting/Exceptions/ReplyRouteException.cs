using System;

namespace Sails.Remoting.Exceptions;

public class ReplyRouteException : Exception
{
    public ReplyRouteException() : base()
    {
    }

    public ReplyRouteException(string message) : base(message)
    {
    }

    public ReplyRouteException(string message, Exception innerException) : base(message, innerException)
    {
    }
}
