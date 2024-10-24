using EnsureThat;
using Substrate.NET.Schnorrkel;
using Substrate.NetApi.Model.Types;

namespace Substrate.Gear.Client.Model.Types;

public static class AccountExtensions
{
    /// <summary>
    /// Returns a public key of the account.
    /// </summary>
    /// <param name="account"></param>
    /// <returns></returns>
    public static PublicKey GetPublicKey(this Account account)
    {
        EnsureArg.IsNotNull(account, nameof(account));
        EnsureArg.HasItems(account.Bytes, nameof(account.Bytes));

        return new PublicKey(account.Bytes);
    }
}
