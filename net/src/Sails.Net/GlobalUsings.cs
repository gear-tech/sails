global using CodeChangedEventData = Substrate.NetApi.Model.Types.Base.BaseTuple<
    Substrate.Gear.Api.Generated.Model.gprimitives.CodeId,
    Substrate.Gear.Api.Generated.Model.gear_common.@event.EnumCodeChangeKind>;
global using CodeId = Substrate.Gear.Api.Generated.Model.gprimitives.CodeId;
global using EnumGearEvent = Substrate.Gear.Api.Generated.Model.pallet_gear.pallet.EnumEvent;
global using EnumSystemEvent = Substrate.Gear.Api.Generated.Model.frame_system.pallet.EnumEvent;
global using ExtrinsicFailedEventData = Substrate.NetApi.Model.Types.Base.BaseTuple<
    Substrate.Gear.Api.Generated.Model.sp_runtime.EnumDispatchError,
    Substrate.Gear.Api.Generated.Model.frame_support.dispatch.DispatchInfo>;
global using GasUnit = Substrate.NetApi.Model.Types.Primitive.U64;
global using GearEvent = Substrate.Gear.Api.Generated.Model.pallet_gear.pallet.Event;
global using MessageQueuedEventData = Substrate.NetApi.Model.Types.Base.BaseTuple<
    Substrate.Gear.Api.Generated.Model.gprimitives.MessageId,
    Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32,
    Substrate.Gear.Api.Generated.Model.gprimitives.ActorId,
    Substrate.Gear.Api.Generated.Model.gear_common.@event.EnumMessageEntry>;
global using SystemEvent = Substrate.Gear.Api.Generated.Model.frame_system.pallet.Event;
global using UserMessageSentEventData = Substrate.NetApi.Model.Types.Base.BaseTuple<
    Substrate.Gear.Api.Generated.Model.gear_core.message.user.UserMessage,
    Substrate.NetApi.Model.Types.Base.BaseOpt<Substrate.NetApi.Model.Types.Primitive.U32>>;
global using ValueUnit = Substrate.NetApi.Model.Types.Primitive.U128;
