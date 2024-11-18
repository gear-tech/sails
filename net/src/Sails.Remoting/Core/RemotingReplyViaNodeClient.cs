using System;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Sails.Remoting.Abstractions.Core;
using StreamJsonRpc;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.frame_system;
using Substrate.Gear.Api.Generated.Model.gear_core.message.user;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Api.Generated.Model.vara_runtime;
using Substrate.Gear.Client;
using Substrate.Gear.Client.Model.Rpc;
using Substrate.Gear.Client.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;
using EnumGearEvent = Substrate.Gear.Api.Generated.Model.pallet_gear.pallet.EnumEvent;
using GearEvent = Substrate.Gear.Api.Generated.Model.pallet_gear.pallet.Event;
using MessageQueuedEventData = Substrate.NetApi.Model.Types.Base.BaseTuple<
    Substrate.Gear.Api.Generated.Model.gprimitives.MessageId,
    Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32,
    Substrate.Gear.Api.Generated.Model.gprimitives.ActorId,
    Substrate.Gear.Api.Generated.Model.gear_common.@event.EnumMessageEntry>;
using UserMessageSentEventData = Substrate.NetApi.Model.Types.Base.BaseTuple<
    Substrate.Gear.Api.Generated.Model.gear_core.message.user.UserMessage,
    Substrate.NetApi.Model.Types.Base.BaseOpt<Substrate.NetApi.Model.Types.Primitive.U32>>;

namespace Sails.Remoting.Core;

internal sealed class RemotingReplyViaNodeClient<T> : RemotingReply<T>
{
    public static async Task<RemotingReplyViaNodeClient<T>> FromExecutionAsync(
        SubstrateClientExt nodeClient,
        Func<SubstrateClientExt, Task<ExtrinsicInfo>> executeExtrinsic,
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
            var extrinsicInfo = await executeExtrinsic(nodeClient).ConfigureAwait(false);

            var extrinsicBlockEvents = await nodeClient.ListBlockEventsAsync(
                    extrinsicInfo.BlockHash,
                    cancellationToken)
                .ConfigureAwait(false);

            // TODO: Requires checking for System.ExtrinsicFailed event and throwing an exception with
            //       details from it. (type + details)

            var queuedMessageData = extrinsicBlockEvents
                .Where(
                    eventRecord =>
                        eventRecord.Phase.ToBaseEnumRust().Matches(
                            Phase.ApplyExtrinsic,
                            (U32 extrinsicIdxInBlock) => extrinsicIdxInBlock.Value == extrinsicInfo.IndexInBlock))
                .Select(
                    eventRecord => eventRecord.Event.ToBaseEnumRust())
                .SelectIfMatches(
                    RuntimeEvent.Gear,
                    (EnumGearEvent gearEvent) => gearEvent.ToBaseEnumRust())
                .SelectIfMatches(
                    GearEvent.MessageQueued,
                    (MessageQueuedEventData data) => data)
                .SingleOrDefault()
                ?? throw new Exception("TODO: Custom exception. Something terrible happened.");

            var result = new RemotingReplyViaNodeClient<T>(
                nodeClient,
                blocksStream,
                extractResult,
                queuedMessageData);
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
        SubstrateClientExt nodeClient,
        BlocksStream blocksStream,
        Func<MessageQueuedEventData, UserMessage, T> extractResult,
        MessageQueuedEventData queuedMessageData)
    {
        this.nodeClient = nodeClient;
        this.blocksStream = blocksStream;
        this.extractResult = extractResult;
        this.queuedMessageData = queuedMessageData;
        this.replyMessage = null;
    }

    private readonly SubstrateClientExt nodeClient;
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

            this.replyMessage = await this.blocksStream.ReadAllHeadersAsync(cancellationToken)
                .SelectAwait(
                    async blockHeader =>
                        await this.nodeClient.ListBlockEventsAsync(blockHeader.GetBlockHash(), cancellationToken)
                            .ConfigureAwait(false))
                .SelectMany(
                    eventRecords => eventRecords.AsAsyncEnumerable())
                .Select(
                    eventRecord => eventRecord.Event.ToBaseEnumRust())
                .SelectIfMatches(
                    RuntimeEvent.Gear,
                    (EnumGearEvent gearEvent) => gearEvent.ToBaseEnumRust())
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
