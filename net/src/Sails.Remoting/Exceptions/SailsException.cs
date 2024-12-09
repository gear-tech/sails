using System;

namespace Sails.Remoting.Exceptions;

public class SailsException : Exception
{
    public SailsException() : base()
    {
    }

    public SailsException(string message) : base(message)
    {
    }

    public SailsException(string message, Exception innerException) : base(message, innerException)
    {
    }
}
