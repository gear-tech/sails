using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Sails.Remoting.Abstractions;
using Sails.Remoting.Abstractions.Core;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Primitive;

namespace Sails.Remoting;

public sealed class RemotingAction<T>(IRemoting remoting, string programRoute, string actionRoute, params IType[] args)
    : IActivation, IQuery<T>, ICall<T>
    where T : IType, new()
{
    private GasUnit? gasLimit;
    private ValueUnit value = new(0);

    /// <inheritdoc />
    public async Task<IReply<ActorId>> ActivateAsync(
        CodeId codeId,
        IReadOnlyCollection<byte> salt,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(codeId, nameof(codeId));
        EnsureArg.IsNotNull(salt, nameof(salt));

        var encodedPayload = this.EncodePayload(programRoute);

        var remotingReply = await remoting.ActivateAsync(
            codeId,
            salt,
            encodedPayload,
            gasLimit: this.gasLimit,
            value: this.value,
            cancellationToken).ConfigureAwait(false);

        return new DelegatingReply<(ActorId ProgramId, byte[] EncodedReply), ActorId>(remotingReply, res =>
        {
            var p = 0;
            EnsureRoute(res.EncodedReply, ref p, programRoute);
            return res.ProgramId;
        });
    }

    /// <inheritdoc />
    public async Task<IReply<T>> MessageAsync(ActorId programId, CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(programId, nameof(programId));

        var encodedPayload = this.EncodePayload(programRoute, actionRoute);

        var remotingReply = await remoting.MessageAsync(
            programId,
            encodedPayload,
            gasLimit: this.gasLimit,
            value: this.value,
            cancellationToken).ConfigureAwait(false);

        return new DelegatingReply<byte[], T>(remotingReply, this.DecodePayload);
    }

    /// <inheritdoc />
    public async Task<T> QueryAsync(ActorId programId, CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(programId, nameof(programId));

        var encodedPayload = this.EncodePayload(programRoute, actionRoute);

        var replyBytes = await remoting.QueryAsync(
            programId,
            encodedPayload,
            gasLimit: this.gasLimit,
            value: this.value,
            cancellationToken).ConfigureAwait(false);

        return this.DecodePayload(replyBytes);
    }

    /// <inheritdoc />
    public RemotingAction<T> WithGasLimit(GasUnit gasLimit)
    {
        EnsureArg.IsNotNull(gasLimit, nameof(gasLimit));

        this.gasLimit = gasLimit;
        return this;
    }

    /// <inheritdoc />
    public RemotingAction<T> WithValue(ValueUnit value)
    {
        EnsureArg.IsNotNull(value, nameof(value));

        this.value = value;
        return this;
    }

    private byte[] EncodePayload(params string[] routes)
    {
        var byteList = new List<byte>();
        foreach (var route in routes)
        {
            byteList.AddRange(new Str(route).Encode());
        }
        foreach (var arg in args)
        {
            byteList.AddRange(arg.Encode());
        }
        return [.. byteList];
    }

    private T DecodePayload(byte[] bytes)
    {
        var p = 0;
        EnsureRoute(bytes, ref p, programRoute, actionRoute);
        T value = new();
        value.Decode(bytes, ref p);
        return value;
    }

    private static void EnsureRoute(byte[] bytes, ref int p, params string[] routes)
    {
        foreach (var route in routes)
        {
            var str = new Str();
            str.Decode(bytes, ref p);
            if (str != route)
            {
                // TODO: custom invalid route exception
                throw new ArgumentException();
            }
        }
    }

    IActivation IActionBuilder<IActivation>.WithGasLimit(GasUnit gasLimit) => this.WithGasLimit(gasLimit);
    IQuery<T> IActionBuilder<IQuery<T>>.WithGasLimit(GasUnit gasLimit) => this.WithGasLimit(gasLimit);
    ICall<T> IActionBuilder<ICall<T>>.WithGasLimit(GasUnit gasLimit) => this.WithGasLimit(gasLimit);

    IActivation IActionBuilder<IActivation>.WithValue(ValueUnit value) => this.WithValue(value);
    IQuery<T> IActionBuilder<IQuery<T>>.WithValue(ValueUnit value) => this.WithValue(value);
    ICall<T> IActionBuilder<ICall<T>>.WithValue(ValueUnit value) => this.WithValue(value);
}
