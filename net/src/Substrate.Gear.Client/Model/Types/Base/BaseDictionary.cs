using System;
using System.Collections.Generic;
using Newtonsoft.Json;
using Substrate.NetApi;
using Substrate.NetApi.Model.Types;

namespace Substrate.Gear.Client.Model.Types.Base;

/// <summary>
/// Base Dictionary Type
/// </summary>
public class BaseDictionary<TKey, TValue> : IType
        where TKey : IType, new()
        where TValue : IType, new()
{
    /// <summary>
    /// Explicit conversion from Dictionary<TKey, TValue> to BaseDictionary
    /// </summary>
    /// <param name="p"></param>
    public static explicit operator BaseDictionary<TKey, TValue>(Dictionary<TKey, TValue> p) => new(p);

    /// <summary>
    /// Implicit conversion from BaseDictionary to Dictionary<TKey, TValue>
    /// </summary>
    /// <param name="p"></param>
    public static implicit operator Dictionary<TKey, TValue>(BaseDictionary<TKey, TValue> p) => p.Value;

    /// <summary>
    /// BaseDictionary Constructor
    /// </summary>
    public BaseDictionary()
    { }

    /// <summary>
    /// BaseDictionary Constructor
    /// </summary>
    /// <param name="value"></param>
    public BaseDictionary(Dictionary<TKey, TValue> value)
    {
        this.Create(value);
    }

    /// <summary>
    /// BaseDictionary Type Name
    /// </summary>
    /// <returns></returns>
    public virtual string TypeName() => $"Dictionary<{new TKey().TypeName()}, {new TValue().TypeName()}>";

    /// <summary>
    /// BaseDictionary Type Size
    /// </summary>
    public int TypeSize { get; set; }

    /// <summary>
    /// BaseDictionary Bytes
    /// </summary>
    [JsonIgnore]
    public byte[] Bytes { get; internal set; } = [];

    /// <summary>
    /// BaseDictionary Encode
    /// </summary>
    /// <returns></returns>
    public byte[] Encode()
    {
        var result = new List<byte>();
        result.AddRange(new CompactInteger(this.Value.Count).Encode());
        foreach (var kv in this.Value)
        {
            result.AddRange(kv.Key.Encode());
            result.AddRange(kv.Value.Encode());
        }
        return result.ToArray();
    }

    /// <summary>
    /// BaseDictionary Decode
    /// </summary>
    /// <param name="byteArray"></param>
    /// <param name="p"></param>
    public void Decode(byte[] byteArray, ref int p)
    {
        var start = p;

        var length = CompactInteger.Decode(byteArray, ref p);

        var dict = new Dictionary<TKey, TValue>(length);
        for (var i = 0; i < length; i++)
        {
            var key = new TKey();
            key.Decode(byteArray, ref p);
            var val = new TValue();
            val.Decode(byteArray, ref p);
            dict[key] = val;
        }

        this.TypeSize = p - start;

        this.Bytes = new byte[this.TypeSize];
        Array.Copy(byteArray, start, this.Bytes, 0, this.TypeSize);
        this.Value = dict;
    }

    /// <summary>
    /// BaseDictionary Value
    /// </summary>
    public virtual Dictionary<TKey, TValue> Value { get; internal set; } = [];

    /// <summary>
    /// BaseDictionary Create
    /// </summary>
    /// <param name="value"></param>
    public void Create(Dictionary<TKey, TValue> value)
    {
        this.Value = value;
        this.Bytes = this.Encode();
        this.TypeSize = this.Bytes.Length;
    }

    /// <summary>
    /// BaseDictionary Create
    /// </summary>
    /// <param name="str"></param>
    public void Create(string str) => this.Create(Utils.HexToByteArray(str));

    /// <summary>
    /// BaseDictionary Create From Json
    /// </summary>
    /// <param name="str"></param>
    public void CreateFromJson(string str) => this.Create(Utils.HexToByteArray(str));

    /// <summary>
    /// BaseDictionary Create
    /// </summary>
    /// <param name="byteArray"></param>
    public void Create(byte[] byteArray)
    {
        var p = 0;
        this.Decode(byteArray, ref p);
    }

    /// <summary>
    /// BaseDictionary New
    /// </summary>
    /// <returns></returns>
    public IType New() => this;

    /// <inheritdoc/>
    public override string ToString() => JsonConvert.SerializeObject(this);
}
