using System;
using System.Numerics;
using Substrate.NetApi;
using Substrate.NetApi.Model.Types;

namespace Substrate.Gear.Client.Model.Types.Primitive;

/// <summary>
/// U256
/// </summary>
public sealed class U256 : BasePrim<BigInteger>
{
    public override int TypeSize => 32;

    /// <summary>
    /// Explicitly cast a BigInteger to a U256
    /// </summary>
    /// <param name="p"></param>
    public static explicit operator U256(BigInteger p)
    {
        return new U256(p);
    }

    /// <summary>
    /// Implicitly cast a U256 to a BigInteger
    /// </summary>
    /// <param name="p"></param>
    public static implicit operator BigInteger(U256 p)
    {
        return p.Value;
    }

    /// <summary>
    /// U256 Constructor
    /// </summary>
    public U256()
    {
    }

    /// <summary>
    /// U256 Constructor
    /// </summary>
    public U256(BigInteger value)
    {
        this.Create(value);
    }

    public override string TypeName() => nameof(U256);

    public override byte[] Encode() => this.Bytes;

    public override void CreateFromJson(string str)
    {
        var array = Utils.HexToByteArray(str, evenLeftZeroPad: true);
        Array.Reverse(array);
        var array2 = new byte[this.TypeSize];
        array.CopyTo(array2, 0);
        this.Create(array2);
    }

    public override void Create(byte[] byteArray)
    {
        if (byteArray.Length < this.TypeSize)
        {
            var array = new byte[this.TypeSize];
            byteArray.CopyTo(array, 0);
            byteArray = array;
        }
        else
        {
            if (byteArray.Length != this.TypeSize)
            {
                throw new NotSupportedException($"Wrong byte array size for {this.TypeName()}, max. {this.TypeSize} bytes!");
            }

            var array2 = new byte[byteArray.Length + 2];
            byteArray.CopyTo(array2, 0);
            array2[byteArray.Length - 1] = 0;
        }

        this.Bytes = byteArray;
        this.Value = new BigInteger(byteArray);
    }

    public override void Create(BigInteger value)
    {
        if (value.Sign < 0)
        {
            throw new InvalidOperationException($"Unable to create a {this.TypeName()} instance while value is negative");
        }

        var array = value.ToByteArray();
        if (array.Length > this.TypeSize)
        {
            throw new NotSupportedException($"Wrong byte array size for {this.TypeName()}, max. {this.TypeSize} bytes!");
        }

        var array2 = new byte[this.TypeSize];
        array.CopyTo(array2, 0);
        this.Bytes = array2;
        this.Value = value;
    }
}
