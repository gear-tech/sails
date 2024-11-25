namespace Sails.Remoting.Abstractions;

public interface IActionBuilder<TAction>
{
    /// <summary>
    /// Sets the gas limit of the transaction manually.
    /// </summary>
    /// <param name="gasLimit">Gas limit.</param>
    /// <returns></returns>
    TAction WithGasLimit(GasUnit gasLimit);

    /// <summary>
    /// Sets the value of the message.
    /// </summary>
    /// <param name="value">Value</param>
    /// <returns></returns>
    TAction WithValue(ValueUnit value);
}
