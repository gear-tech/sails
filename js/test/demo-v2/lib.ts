import { GearApi, HexString } from "@gear-js/api";
import { ActorId, H160, H256, NonZeroU32, NonZeroU8, QueryBuilderWithHeader, TransactionBuilderWithHeader, TypeResolver, ZERO_ADDRESS } from '../..';
import { InterfaceId, SailsMessageHeader } from "sails-js-parser-idl-v2";
import { IStructField } from "sails-js-types";

export class DemoClient {
    private _typeResolver: TypeResolver;
    constructor(
        public api: GearApi,
        private _programId?: `0x${string}`,
    ) {
        this._typeResolver = new TypeResolver([]);
    }

    private get registry() {
        return this._typeResolver.registry;
    }

    public get programId(): HexString {
        if (!this._programId) throw new Error("Program ID is not set");
        return this._programId;
    }

    /**
     * Program constructor (called once at the very beginning of the program lifetime)
     */
    public defaultCtorFromCode(code: Uint8Array | Buffer | HexString): TransactionBuilderWithHeader<null> {
        const builder = new TransactionBuilderWithHeader<null>(
            this.api,
            this.registry,
            "upload_program",
            SailsMessageHeader.v1(InterfaceId.zero(), 0, 0),
            null,
            null,
            this._typeResolver.getTypeDeclString("String"),
            code,
        );
        this._programId = builder.programId;
        return builder;
    }

    /**
     * Program constructor (called once at the very beginning of the program lifetime)
     */
    public defaultCtorFromCodeId(codeId: `0x${string}`): TransactionBuilderWithHeader<null> {
        const builder = new TransactionBuilderWithHeader<null>(
            this.api,
            this.registry,
            "create_program",
            SailsMessageHeader.v1(InterfaceId.zero(), 0, 0),
            null,
            null,
            this._typeResolver.getTypeDeclString("String"),
            codeId,
        );
        this._programId = builder.programId;
        return builder;
    }

    /**
     * Another program constructor (called once at the very beginning of the program lifetime)
     */
    public newCtorFromCode(code: Uint8Array | Buffer | HexString, counter: number | null, dog_position: [number, number] | null): TransactionBuilderWithHeader<null> {
        const builder = new TransactionBuilderWithHeader<null>(
            this.api,
            this.registry,
            "upload_program",
            SailsMessageHeader.v1(InterfaceId.zero(), 1, 0),
            [counter, dog_position],
            this._typeResolver.getTypeDeclString({ "kind": "tuple", "types": [{ "kind": "named", "name": "Option", "generics": ["u32"] }, { "kind": "named", "name": "Option", "generics": [{ "kind": "tuple", "types": ["i32", "i32"] }] }] }),
            this._typeResolver.getTypeDeclString("String"),
            code,
        );
        this._programId = builder.programId;
        return builder;
    }

    /**
     * Another program constructor (called once at the very beginning of the program lifetime)
     */
    public newCtorFromCodeId(codeId: `0x${string}`, counter: number | null, dog_position: [number, number] | null): TransactionBuilderWithHeader<null> {
        const builder = new TransactionBuilderWithHeader<null>(
            this.api,
            this.registry,
            "create_program",
            SailsMessageHeader.v1(InterfaceId.zero(), 1, 0),
            [counter, dog_position],
            this._typeResolver.getTypeDeclString({ "kind": "tuple", "types": [{ "kind": "named", "name": "Option", "generics": ["u32"] }, { "kind": "named", "name": "Option", "generics": [{ "kind": "tuple", "types": ["i32", "i32"] }] }] }),
            this._typeResolver.getTypeDeclString("String"),
            codeId,
        );
        this._programId = builder.programId;
        return builder;
    }

    public get pingPong(): PingPong {
        return new PingPong(this.api, this.programId, 1);
    }

    public get counter(): Counter {
        return new Counter(this.api, this.programId, 2);
    }

    public get dog(): Dog {
        return new Dog(this.api, this.programId, 3);
    }

    public get references(): References {
        return new References(this.api, this.programId, 4);
    }

    public get thisThat(): ThisThat {
        return new ThisThat(this.api, this.programId, 5);
    }

    public get valueFee(): ValueFee {
        return new ValueFee(this.api, this.programId, 6);
    }

    public get chaos(): Chaos {
        return new Chaos(this.api, this.programId, 7);
    }

    public get chain(): Chain {
        return new Chain(this.api, this.programId, 8);
    }
}

export class PingPong {
    private _typeResolver: TypeResolver;
    constructor(
        private _api: GearApi,
        private _programId: HexString,
        private _routeIdx: number = 0,
    ) {
        this._typeResolver = new TypeResolver([]);
    }
    private get registry() {
        return this._typeResolver.registry;
    }
    public get interfaceId(): InterfaceId {
        return InterfaceId.from("0x6d0eb40dde4038f7");
    }
    public ping(input: string): TransactionBuilderWithHeader<{ ok: string } | { err: string }> {
        return new TransactionBuilderWithHeader<{ ok: string } | { err: string }>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 0, this._routeIdx),
            input,
            this._typeResolver.getTypeDeclString("String"),
            this._typeResolver.getTypeDeclString({ "kind": "named", "name": "Result", "generics": ["String", "String"] }),
            this._programId,
        );
    }
}

export class Counter {
    private _typeResolver: TypeResolver;
    constructor(
        private _api: GearApi,
        private _programId: HexString,
        private _routeIdx: number = 0,
    ) {
        this._typeResolver = new TypeResolver([]);
    }
    private get registry() {
        return this._typeResolver.registry;
    }
    public get interfaceId(): InterfaceId {
        return InterfaceId.from("0x579d6daba41b7d82");
    }
    /**
     * Add a value to the counter
     */
    public add(value: number): TransactionBuilderWithHeader<number> {
        return new TransactionBuilderWithHeader<number>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 0, this._routeIdx),
            value,
            this._typeResolver.getTypeDeclString("u32"),
            this._typeResolver.getTypeDeclString("u32"),
            this._programId,
        );
    }

    /**
     * Substract a value from the counter
     */
    public sub(value: number): TransactionBuilderWithHeader<number> {
        return new TransactionBuilderWithHeader<number>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 1, this._routeIdx),
            value,
            this._typeResolver.getTypeDeclString("u32"),
            this._typeResolver.getTypeDeclString("u32"),
            this._programId,
        );
    }

    /**
     * Get the current value
     */
    public value(): QueryBuilderWithHeader<number> {
        return new QueryBuilderWithHeader<number>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 2, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("u32"),
        );
    }

    /**
     * Emitted when a new value is added to the counter
     */
    public subscribeToAddedEvent<T = number>(callback: (eventData: T) => void | Promise<void>): Promise<() => void> {
        const interfaceIdu64 = this.interfaceId.asU64();
        const eventFields = { "fields": [{ "type": "u32" }] }.fields as IStructField[];
        const typeStr = this._typeResolver.getStructDef(eventFields, {}, true);
        return this._api.gearEvents.subscribeToGearEvent("UserMessageSent", ({ data: { message } }) => {
            if (!message.source.eq(this._programId)) return;
            if (!message.destination.eq(ZERO_ADDRESS)) return;

            const { ok, header } = SailsMessageHeader.tryFromBytes(message.payload);
            if (ok && header.interfaceId.asU64() === interfaceIdu64 && header.entryId === 0) {
                callback(this.registry.createType(`([u8; 16], ${typeStr})`, message.payload)[1].toJSON() as T);
            }
        });
    }

    /**
     * Emitted when a value is subtracted from the counter
     */
    public subscribeToSubtractedEvent<T = number>(callback: (eventData: T) => void | Promise<void>): Promise<() => void> {
        const interfaceIdu64 = this.interfaceId.asU64();
        const eventFields = { "fields": [{ "type": "u32" }] }.fields as IStructField[];
        const typeStr = this._typeResolver.getStructDef(eventFields, {}, true);
        return this._api.gearEvents.subscribeToGearEvent("UserMessageSent", ({ data: { message } }) => {
            if (!message.source.eq(this._programId)) return;
            if (!message.destination.eq(ZERO_ADDRESS)) return;

            const { ok, header } = SailsMessageHeader.tryFromBytes(message.payload);
            if (ok && header.interfaceId.asU64() === interfaceIdu64 && header.entryId === 1) {
                callback(this.registry.createType(`([u8; 16], ${typeStr})`, message.payload)[1].toJSON() as T);
            }
        });
    }
}

export class MammalService {
    private _typeResolver: TypeResolver;
    constructor(
        private _api: GearApi,
        private _programId: HexString,
        private _routeIdx: number = 0,
    ) {
        this._typeResolver = new TypeResolver([]);
    }
    private get registry() {
        return this._typeResolver.registry;
    }
    public get interfaceId(): InterfaceId {
        return InterfaceId.from("0xff6b93e1961026fe");
    }
    public makeSound(): TransactionBuilderWithHeader<string> {
        return new TransactionBuilderWithHeader<string>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 0, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("String"),
            this._programId,
        );
    }

    public avgWeight(): QueryBuilderWithHeader<number> {
        return new QueryBuilderWithHeader<number>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 1, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("u32"),
        );
    }
}

export class WalkerService {
    private _typeResolver: TypeResolver;
    constructor(
        private _api: GearApi,
        private _programId: HexString,
        private _routeIdx: number = 0,
    ) {
        this._typeResolver = new TypeResolver([]);
    }
    private get registry() {
        return this._typeResolver.registry;
    }
    public get interfaceId(): InterfaceId {
        return InterfaceId.from("0xee1536b55170bf0a");
    }
    public walk(dx: number, dy: number): TransactionBuilderWithHeader<null> {
        return new TransactionBuilderWithHeader<null>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 0, this._routeIdx),
            [dx, dy],
            this._typeResolver.getTypeDeclString({ "kind": "tuple", "types": ["i32", "i32"] }),
            this._typeResolver.getTypeDeclString("()"),
            this._programId,
        );
    }

    public position(): QueryBuilderWithHeader<[number, number]> {
        return new QueryBuilderWithHeader<[number, number]>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 1, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString({ "kind": "tuple", "types": ["i32", "i32"] }),
        );
    }

    public subscribeToWalkedEvent<T = { $from: [number, number]; to: [number, number] }>(callback: (eventData: T) => void | Promise<void>): Promise<() => void> {
        const interfaceIdu64 = this.interfaceId.asU64();
        const eventFields = { "fields": [{ "name": "from", "type": { "kind": "tuple", "types": ["i32", "i32"] } }, { "name": "to", "type": { "kind": "tuple", "types": ["i32", "i32"] } }] }.fields as IStructField[];
        const typeStr = this._typeResolver.getStructDef(eventFields, {}, true);
        return this._api.gearEvents.subscribeToGearEvent("UserMessageSent", ({ data: { message } }) => {
            if (!message.source.eq(this._programId)) return;
            if (!message.destination.eq(ZERO_ADDRESS)) return;

            const { ok, header } = SailsMessageHeader.tryFromBytes(message.payload);
            if (ok && header.interfaceId.asU64() === interfaceIdu64 && header.entryId === 0) {
                callback(this.registry.createType(`([u8; 16], ${typeStr})`, message.payload)[1].toJSON() as T);
            }
        });
    }
}

export class Dog {
    private _typeResolver: TypeResolver;
    constructor(
        private _api: GearApi,
        private _programId: HexString,
        private _routeIdx: number = 0,
    ) {
        this._typeResolver = new TypeResolver([]);
    }
    private get registry() {
        return this._typeResolver.registry;
    }
    public get interfaceId(): InterfaceId {
        return InterfaceId.from("0x18666e67a21917a1");
    }
    public get mammalService(): MammalService {
        return new MammalService(this._api, this._programId, this._routeIdx);
    }

    public get walkerService(): WalkerService {
        return new WalkerService(this._api, this._programId, this._routeIdx);
    }

    public makeSound(): TransactionBuilderWithHeader<string> {
        return new TransactionBuilderWithHeader<string>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 0, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("String"),
            this._programId,
        );
    }

    public subscribeToBarkedEvent<T = null>(callback: (eventData: T) => void | Promise<void>): Promise<() => void> {
        const interfaceIdu64 = this.interfaceId.asU64();
        const eventFields = { "fields": [] }.fields as IStructField[];
        const typeStr = this._typeResolver.getStructDef(eventFields, {}, true);
        return this._api.gearEvents.subscribeToGearEvent("UserMessageSent", ({ data: { message } }) => {
            if (!message.source.eq(this._programId)) return;
            if (!message.destination.eq(ZERO_ADDRESS)) return;

            const { ok, header } = SailsMessageHeader.tryFromBytes(message.payload);
            if (ok && header.interfaceId.asU64() === interfaceIdu64 && header.entryId === 0) {
                callback(this.registry.createType(`([u8; 16], ${typeStr})`, message.payload)[1].toJSON() as T);
            }
        });
    }
}

export type ReferenceCount = number;

export class References {
    private _typeResolver: TypeResolver;
    constructor(
        private _api: GearApi,
        private _programId: HexString,
        private _routeIdx: number = 0,
    ) {
        this._typeResolver = new TypeResolver([{ "name": "ReferenceCount", "kind": "struct", "fields": [{ "type": "u32" }] }]);
    }
    private get registry() {
        return this._typeResolver.registry;
    }
    public get interfaceId(): InterfaceId {
        return InterfaceId.from("0x8a0f8abe176d75b9");
    }
    public add(v: number): TransactionBuilderWithHeader<number> {
        return new TransactionBuilderWithHeader<number>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 0, this._routeIdx),
            v,
            this._typeResolver.getTypeDeclString("u32"),
            this._typeResolver.getTypeDeclString("u32"),
            this._programId,
        );
    }

    public addByte(byte: number): TransactionBuilderWithHeader<number[]> {
        return new TransactionBuilderWithHeader<number[]>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 1, this._routeIdx),
            byte,
            this._typeResolver.getTypeDeclString("u8"),
            this._typeResolver.getTypeDeclString({ "kind": "slice", "item": "u8" }),
            this._programId,
        );
    }

    public guessNum($number: number): TransactionBuilderWithHeader<{ ok: string } | { err: string }> {
        return new TransactionBuilderWithHeader<{ ok: string } | { err: string }>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 2, this._routeIdx),
            $number,
            this._typeResolver.getTypeDeclString("u8"),
            this._typeResolver.getTypeDeclString({ "kind": "named", "name": "Result", "generics": ["String", "String"] }),
            this._programId,
        );
    }

    public incr(): TransactionBuilderWithHeader<ReferenceCount> {
        return new TransactionBuilderWithHeader<ReferenceCount>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 3, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString({ "kind": "named", "name": "ReferenceCount" }),
            this._programId,
        );
    }

    public setNum($number: number): TransactionBuilderWithHeader<{ ok: null } | { err: string }> {
        return new TransactionBuilderWithHeader<{ ok: null } | { err: string }>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 4, this._routeIdx),
            $number,
            this._typeResolver.getTypeDeclString("u8"),
            this._typeResolver.getTypeDeclString({ "kind": "named", "name": "Result", "generics": ["()", "String"] }),
            this._programId,
        );
    }

    public baked(): QueryBuilderWithHeader<string> {
        return new QueryBuilderWithHeader<string>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 5, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("String"),
        );
    }

    public lastByte(): QueryBuilderWithHeader<number | null> {
        return new QueryBuilderWithHeader<number | null>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 6, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString({ "kind": "named", "name": "Option", "generics": ["u8"] }),
        );
    }

    public message(): QueryBuilderWithHeader<string | null> {
        return new QueryBuilderWithHeader<string | null>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 7, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString({ "kind": "named", "name": "Option", "generics": ["String"] }),
        );
    }
}

export interface DoThatParam { p1: NonZeroU32; p2: ActorId; p3: ManyVariants }

export type ManyVariants = { One: null } | { Two: number } | { Three: bigint | null } | { Four: { a: number; b: number | null } } | { Five: [string, H256] } | { Six: [number] };

export type ManyVariantsReply = "One" | "Two" | "Three" | "Four" | "Five" | "Six";

export type TupleStruct = boolean;

export class ThisThat {
    private _typeResolver: TypeResolver;
    constructor(
        private _api: GearApi,
        private _programId: HexString,
        private _routeIdx: number = 0,
    ) {
        this._typeResolver = new TypeResolver([{ "name": "DoThatParam", "kind": "struct", "fields": [{ "name": "p1", "type": { "kind": "named", "name": "NonZeroU32" } }, { "name": "p2", "type": "ActorId" }, { "name": "p3", "type": { "kind": "named", "name": "ManyVariants" } }] }, { "name": "ManyVariants", "kind": "enum", "variants": [{ "name": "One", "fields": [] }, { "name": "Two", "fields": [{ "type": "u32" }] }, { "name": "Three", "fields": [{ "type": { "kind": "named", "name": "Option", "generics": ["U256"] } }] }, { "name": "Four", "fields": [{ "name": "a", "type": "u32" }, { "name": "b", "type": { "kind": "named", "name": "Option", "generics": ["u16"] } }] }, { "name": "Five", "fields": [{ "type": "String" }, { "type": "H256" }] }, { "name": "Six", "fields": [{ "type": { "kind": "tuple", "types": ["u32"] } }] }] }, { "name": "ManyVariantsReply", "kind": "enum", "variants": [{ "name": "One", "fields": [] }, { "name": "Two", "fields": [] }, { "name": "Three", "fields": [] }, { "name": "Four", "fields": [] }, { "name": "Five", "fields": [] }, { "name": "Six", "fields": [] }] }, { "name": "NonZeroU32", "kind": "struct", "fields": [{ "type": "u32" }] }, { "name": "NonZeroU8", "kind": "struct", "fields": [{ "type": "u8" }] }, { "name": "TupleStruct", "kind": "struct", "fields": [{ "type": "bool" }] }]);
    }
    private get registry() {
        return this._typeResolver.registry;
    }
    public get interfaceId(): InterfaceId {
        return InterfaceId.from("0x381e13fdd02d675f");
    }
    public doThat(param: DoThatParam): TransactionBuilderWithHeader<{ ok: [ActorId, NonZeroU32, ManyVariantsReply] } | { err: [string] }> {
        return new TransactionBuilderWithHeader<{ ok: [ActorId, NonZeroU32, ManyVariantsReply] } | { err: [string] }>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 0, this._routeIdx),
            param,
            this._typeResolver.getTypeDeclString({ "kind": "named", "name": "DoThatParam" }),
            this._typeResolver.getTypeDeclString({ "kind": "named", "name": "Result", "generics": [{ "kind": "tuple", "types": ["ActorId", { "kind": "named", "name": "NonZeroU32" }, { "kind": "named", "name": "ManyVariantsReply" }] }, { "kind": "tuple", "types": ["String"] }] }),
            this._programId,
        );
    }

    public doThis(p1: number, p2: string, p3: [H160 | null, NonZeroU8], p4: TupleStruct): TransactionBuilderWithHeader<[string, number]> {
        return new TransactionBuilderWithHeader<[string, number]>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 1, this._routeIdx),
            [p1, p2, p3, p4],
            this._typeResolver.getTypeDeclString({ "kind": "tuple", "types": ["u32", "String", { "kind": "tuple", "types": [{ "kind": "named", "name": "Option", "generics": ["H160"] }, { "kind": "named", "name": "NonZeroU8" }] }, { "kind": "named", "name": "TupleStruct" }] }),
            this._typeResolver.getTypeDeclString({ "kind": "tuple", "types": ["String", "u32"] }),
            this._programId,
        );
    }

    public noop(): TransactionBuilderWithHeader<null> {
        return new TransactionBuilderWithHeader<null>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 2, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("()"),
            this._programId,
        );
    }

    public that(): QueryBuilderWithHeader<{ ok: string } | { err: string }> {
        return new QueryBuilderWithHeader<{ ok: string } | { err: string }>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 3, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString({ "kind": "named", "name": "Result", "generics": ["String", "String"] }),
        );
    }

    public $this(): QueryBuilderWithHeader<number> {
        return new QueryBuilderWithHeader<number>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 4, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("u32"),
        );
    }
}

export class ValueFee {
    private _typeResolver: TypeResolver;
    constructor(
        private _api: GearApi,
        private _programId: HexString,
        private _routeIdx: number = 0,
    ) {
        this._typeResolver = new TypeResolver([]);
    }
    private get registry() {
        return this._typeResolver.registry;
    }
    public get interfaceId(): InterfaceId {
        return InterfaceId.from("0x41c1080b4e1e8dc5");
    }
    /**
     * Return flag if fee taken and remain value,
     * using special type `CommandReply<T>`
     */
    public doSomethingAndTakeFee(): TransactionBuilderWithHeader<boolean> {
        return new TransactionBuilderWithHeader<boolean>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 0, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("bool"),
            this._programId,
        );
    }

    public subscribeToWithheldEvent<T = bigint>(callback: (eventData: T) => void | Promise<void>): Promise<() => void> {
        const interfaceIdu64 = this.interfaceId.asU64();
        const eventFields = { "fields": [{ "type": "u128" }] }.fields as IStructField[];
        const typeStr = this._typeResolver.getStructDef(eventFields, {}, true);
        return this._api.gearEvents.subscribeToGearEvent("UserMessageSent", ({ data: { message } }) => {
            if (!message.source.eq(this._programId)) return;
            if (!message.destination.eq(ZERO_ADDRESS)) return;

            const { ok, header } = SailsMessageHeader.tryFromBytes(message.payload);
            if (ok && header.interfaceId.asU64() === interfaceIdu64 && header.entryId === 0) {
                callback(this.registry.createType(`([u8; 16], ${typeStr})`, message.payload)[1].toJSON() as T);
            }
        });
    }
}

export class Chaos {
    private _typeResolver: TypeResolver;
    constructor(
        private _api: GearApi,
        private _programId: HexString,
        private _routeIdx: number = 0,
    ) {
        this._typeResolver = new TypeResolver([]);
    }
    private get registry() {
        return this._typeResolver.registry;
    }
    public get interfaceId(): InterfaceId {
        return InterfaceId.from("0xf0c8c80dfabf72d5");
    }
    public panicAfterWait(): QueryBuilderWithHeader<null> {
        return new QueryBuilderWithHeader<null>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 0, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("()"),
        );
    }

    public replyHookCounter(): QueryBuilderWithHeader<number> {
        return new QueryBuilderWithHeader<number>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 1, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("u32"),
        );
    }

    public timeoutWait(): QueryBuilderWithHeader<null> {
        return new QueryBuilderWithHeader<null>(
            this._api,
            this.registry,
            this._programId,
            SailsMessageHeader.v1(this.interfaceId, 2, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("()"),
        );
    }
}

export class Chain {
    private _typeResolver: TypeResolver;
    constructor(
        private _api: GearApi,
        private _programId: HexString,
        private _routeIdx: number = 0,
    ) {
        this._typeResolver = new TypeResolver([]);
    }
    private get registry() {
        return this._typeResolver.registry;
    }
    public get interfaceId(): InterfaceId {
        return InterfaceId.from("0xd422c66e6021e0f9");
    }
    public get dog(): Dog {
        return new Dog(this._api, this._programId, this._routeIdx);
    }

    public makeSound(): TransactionBuilderWithHeader<string> {
        return new TransactionBuilderWithHeader<string>(
            this._api,
            this.registry,
            "send_message",
            SailsMessageHeader.v1(this.interfaceId, 0, this._routeIdx),
            null,
            null,
            this._typeResolver.getTypeDeclString("String"),
            this._programId,
        );
    }
}
