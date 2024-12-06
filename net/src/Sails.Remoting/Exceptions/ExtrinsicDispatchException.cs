using System;
using Substrate.Gear.Api.Generated.Model.frame_support.dispatch;
using Substrate.Gear.Api.Generated.Model.sp_runtime;

namespace Sails.Remoting.Exceptions;

public class ExtrinsicDispatchException : Exception
{
    public EnumDispatchError? DispatchError { get; }
    public DispatchInfo? DispatchInfo { get; }

    public ExtrinsicDispatchException(string message, ExtrinsicFailedEventData? eventData) : base(message)
    {
        this.DispatchError = eventData?.Value[0] as EnumDispatchError;
        this.DispatchInfo = eventData?.Value[1] as DispatchInfo;
    }

    public ExtrinsicDispatchException(string message, EnumDispatchError? dispatchError, DispatchInfo? dispatchInfo)
        : base(message)
    {
        this.DispatchError = dispatchError;
        this.DispatchInfo = dispatchInfo;
    }

    public ExtrinsicDispatchException() : base()
    {
    }

    public ExtrinsicDispatchException(string message) : base(message)
    {
    }

    public ExtrinsicDispatchException(string message, Exception innerException) : base(message, innerException)
    {
    }
}
