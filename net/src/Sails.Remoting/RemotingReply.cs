using System;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.VisualStudio.Threading;
using Sails.Remoting.Abstractions;
using Substrate.NetApi.Model.Types;

namespace Sails.Remoting;

public class RemotingReply<TResult, T> : IReply<T> where T : IType, new()
{
    private readonly AsyncLazy<TResult> lazyTask;
    private readonly Func<TResult, T> map;


    public RemotingReply(Task<TResult> task, Func<TResult, T> map)
    {
        // TODO: Should use JoinableTaskFactory?
#pragma warning disable VSTHRD012 // Provide JoinableTaskFactory where allowed
#pragma warning disable VSTHRD003 // Avoid awaiting foreign Tasks
        this.lazyTask = new AsyncLazy<TResult>(() => task);
#pragma warning restore VSTHRD003 // Avoid awaiting foreign Tasks
#pragma warning restore VSTHRD012 // Provide JoinableTaskFactory where allowed
        this.map = map;
    }

    /// <inheritdoc />
    public async Task<T> ReceiveAsync(CancellationToken cancellationToken)
    {
        var result = await this.lazyTask.GetValueAsync(cancellationToken).ConfigureAwait(false);
        return this.map(result);
    }
}
