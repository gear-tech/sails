using System;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;
using Sails.Remoting.Abstractions;
using StreamJsonRpc;
using Substrate.Gear.Api.Generated;
using Substrate.Gear.Api.Generated.Model.frame_system;
using Substrate.Gear.Api.Generated.Model.gear_core.message.user;
using Substrate.Gear.Api.Generated.Model.gprimitives;
using Substrate.Gear.Api.Generated.Model.vara_runtime;
using Substrate.Gear.Client;
using Substrate.Gear.Client.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;
using EnumGearEvent = Substrate.Gear.Api.Generated.Model.pallet_gear.pallet.EnumEvent;
using ExtrinsicInfo = Substrate.Gear.Client.ExtrinsicInfo;
using GearEvent = Substrate.Gear.Api.Generated.Model.pallet_gear.pallet.Event;
using MessageQueuedEventData = Substrate.NetApi.Model.Types.Base.BaseTuple<
    Substrate.Gear.Api.Generated.Model.gprimitives.MessageId,
    Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32,
    Substrate.Gear.Api.Generated.Model.gprimitives.ActorId,
    Substrate.Gear.Api.Generated.Model.gear_common.@event.EnumMessageEntry>;
using UserMessageSentEventData = Substrate.NetApi.Model.Types.Base.BaseTuple<
    Substrate.Gear.Api.Generated.Model.gear_core.message.user.UserMessage,
    Substrate.NetApi.Model.Types.Base.BaseOpt<Substrate.NetApi.Model.Types.Primitive.U32>>;

namespace Sails.Remoting;

internal sealed class ActivationResultViaNodeClient : ActivationResult
{
    public static async Task<ActivationResultViaNodeClient> FromExecutionAsync(
        SubstrateClientExt nodeClient,
        Func<SubstrateClientExt, Task<ExtrinsicInfo>> executeExtrinsic,
        CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNull(nodeClient, nameof(nodeClient));
        EnsureArg.IsNotNull(executeExtrinsic, nameof(executeExtrinsic));

        var blocksStream = await nodeClient.GetAllBlocksStreamAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            var extrinsicInfo = await executeExtrinsic(nodeClient).ConfigureAwait(false);

            var extrinsicBlockEvents = await nodeClient.ListBlockEventsAsync(
                    extrinsicInfo.BlockHash,
                    cancellationToken)
                .ConfigureAwait(false);

            // TODO: Requires checking for System.ExtrinsicFailed event and throwing an exception with
            //       details from it. (type + details)

            var messageQueuedEventData = extrinsicBlockEvents
                .Where(
                    eventRecord =>
                        eventRecord.Phase.Matches(
                            Phase.ApplyExtrinsic,
                            (U32 extrinsicIdxInBlock) => extrinsicIdxInBlock.Value == extrinsicInfo.IndexInBlock))
                .Select(
                    eventRecord => eventRecord.Event)
                .SelectIfMatches(
                    RuntimeEvent.Gear,
                    (EnumGearEvent gearEvent) => gearEvent)
                .SelectIfMatches(
                    GearEvent.MessageQueued,
                    (MessageQueuedEventData data) => data)
                .SingleOrDefault()
                ?? throw new Exception("TODO: Custom exception. Something terrible happened.");

            var result = new ActivationResultViaNodeClient(nodeClient, blocksStream, messageQueuedEventData);
            blocksStream = null;
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

    private ActivationResultViaNodeClient(
        SubstrateClientExt nodeClient,
        BlocksStream blocksStream,
        MessageQueuedEventData messageQueuedEventData)
    {
        this.nodeClient = nodeClient;
        this.blocksStream = blocksStream;
        this.messageQueuedEventData = messageQueuedEventData;
    }

    private readonly SubstrateClientExt nodeClient;
    private readonly BlocksStream blocksStream;
    private readonly MessageQueuedEventData messageQueuedEventData;

    protected override ValueTask DisposeCoreAsync()
        => this.blocksStream.DisposeAsync();

    public override async Task<(ActorId ProgramId, byte[] EncodedPayload)> ReadReplyAsync(CancellationToken cancellationToken)
    {
        var queuedMessageId = (MessageId)this.messageQueuedEventData.Value[0];
        var activatedProgramId = (ActorId)this.messageQueuedEventData.Value[2];

        var replyMessage = await this.blocksStream.ReadAllHeadersAsync(cancellationToken)
            .SelectAwait(
                async blockHeader =>
                    await this.nodeClient.ListBlockEventsAsync(blockHeader.Number, cancellationToken) // TODO: It is weird block header doesn't contain hash.
                        .ConfigureAwait(false))
            .SelectMany(
                eventRecords => eventRecords.AsAsyncEnumerable())
            .Select(
                eventRecord => eventRecord.Event)
            .SelectIfMatches(
                RuntimeEvent.Gear,
                (EnumGearEvent gearEvent) => gearEvent)
            .SelectIfMatches(
                GearEvent.UserMessageSent,
                (UserMessageSentEventData data) => (UserMessage)data.Value[0])
            .FirstAsync(
                userMessage => userMessage.Details.OptionFlag
                    && userMessage.Details.Value.To.IsEqualTo(queuedMessageId),
                cancellationToken)
            .ConfigureAwait(false);

        var replyPayload = replyMessage.Payload.Value.Value
            .Select(@byte => @byte.Value)
            .ToArray();

        return (activatedProgramId, replyPayload);
    }
}
