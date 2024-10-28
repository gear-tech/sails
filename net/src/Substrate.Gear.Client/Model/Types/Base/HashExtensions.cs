using System.Linq;
using EnsureThat;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.Model.Types.Base;

public static class HashExtensions
{
    /// <summary>
    /// Compares two hashes by their values
    /// </summary>
    /// <param name="left"></param>
    /// <param name="right"></param>
    /// <returns></returns>
    public static bool IsEqualTo(this Hash left, Hash right)
    {
        EnsureArg.IsNotNull(left, nameof(left));
        EnsureArg.IsNotNull(right, nameof(right));

        return left.Bytes.SequenceEqual(left.Bytes);
    }
}
