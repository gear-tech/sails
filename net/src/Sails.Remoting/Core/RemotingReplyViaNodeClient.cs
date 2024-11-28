using System;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Sails.Remoting.Abstractions.Core;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.gear_core.message.user;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Client;
using Substrate.Gear.Client.NetApi.Model.Types.Base;

namespace Sails.Remoting.Core;

internal sealed class RemotingReplyViaNodeClient<T> : RemotingReply<T>
{
    public static async Task<RemotingReplyViaNodeClient<T>> FromExecutionAsync(
        SubstrateClientExt nodeClient,
        Func<SubstrateClientExt, Task<MessageQueuedEventData>> executeExtrinsic,
        Func<MessageQueuedEventData, UserMessage, T> extractResult,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(executeExtrinsic, nameof(executeExtrinsic));
        EnsureArg.IsNotNull(extractResult, nameof(extractResult));

        // TODO: Might need be configurable whether to use best blocks or finalized ones
        var blocksStream = await nodeClient.GetNewBlocksStreamAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            var messageQueuedEventData = await executeExtrinsic(nodeClient).ConfigureAwait(false);

            var result = new RemotingReplyViaNodeClient<T>(
                blocksStream,
                extractResult,
                messageQueuedEventData);
            blocksStream = null; // Prevent disposing the stream in the finally block
            return result;
        }
        finally
        {
            if (blocksStream is not null)
            {
                await blocksStream.DisposeAsync().ConfigureAwait(false);
            }
        }
    }

    private RemotingReplyViaNodeClient(
        BlocksStream blocksStream,
        Func<MessageQueuedEventData, UserMessage, T> extractResult,
        MessageQueuedEventData queuedMessageData)
    {
        this.blocksStream = blocksStream;
        this.extractResult = extractResult;
        this.queuedMessageData = queuedMessageData;
        this.replyMessage = null;
    }

    private BlocksStream? blocksStream;
    private readonly Func<MessageQueuedEventData, UserMessage, T> extractResult;
    private readonly MessageQueuedEventData queuedMessageData;
    private UserMessage? replyMessage;

    protected override async ValueTask DisposeCoreAsync()
    {
        await base.DisposeCoreAsync().ConfigureAwait(false);

        if (this.blocksStream is not null)
        {
            await this.blocksStream.DisposeAsync().ConfigureAwait(false);
            this.blocksStream = null;
        }
    }

    public override async Task<T> ReadAsync(CancellationToken cancellationToken)
    {
        var queuedMessageId = (MessageId)this.queuedMessageData.Value[0];

        if (this.replyMessage is null)
        {
            Ensure.Any.IsNotNull(this.blocksStream, nameof(this.blocksStream));

            this.replyMessage = await this.blocksStream.ReadAllEventsAsync(cancellationToken)
                .SelectGearEvents()
                .SelectIfMatches(
                    GearEvent.UserMessageSent,
                    (UserMessageSentEventData data) => (UserMessage)data.Value[0])
                .FirstAsync(
                    userMessage => userMessage.Details.OptionFlag
                        && userMessage.Details.Value.To.IsEqualTo(queuedMessageId),
                    cancellationToken)
                .ConfigureAwait(false);

            await this.blocksStream.DisposeAsync().ConfigureAwait(false);
            this.blocksStream = null;
        }

        return this.extractResult(this.queuedMessageData, this.replyMessage);
    }
}
