namespace Substrate.Gear.Client.Exceptions;

public class GearException : Exception
{
    public GearException()
    {
    }

    public GearException(string message)
        : base(message)
    {
    }

    public GearException(string message, Exception innerException)
        : base(message, innerException)
    {
    }
}
