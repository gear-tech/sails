import type { IInterfaceId } from './idl-v2-types';

export type ResolvedEntry =
  | {
      kind: 'command' | 'query';
      service: string;
      fn: string;
      route: string;
      interfaceId: IInterfaceId;
      entryId: number;
      route_idx: number;
    }
  | {
      kind: 'event';
      service: string;
      event: string;
      route: string;
      interfaceId: IInterfaceId;
      entryId: number;
      route_idx: number;
    }
  | {
      kind: 'ctor';
      ctor: string;
      interfaceId: IInterfaceId;
      entryId: number;
      route_idx: number;
    };

export type DecodeReason =
  | 'too-short'
  | 'no-magic'
  | 'bad-version'
  | 'bad-reserved'
  | 'bad-hlen'
  | 'no-service'
  | 'no-entry'
  | 'ambiguous-route'
  | 'decode-failed'
  | 'trailing-bytes'
  | 'entry-mismatch'
  | 'no-throws-type';

export type DecodedUnknown = {
  kind: 'unknown';
  reason: DecodeReason;
  detail?: string;
  consumedLen?: number;
};

export type DecodedCall = { kind: 'call'; entry: ResolvedEntry; args: Record<string, unknown> };
export type DecodedReply = { kind: 'reply'; entry: ResolvedEntry; result: unknown };
export type DecodedError = { kind: 'error'; entry: ResolvedEntry; error: unknown };
export type DecodedEvent = { kind: 'event'; entry: ResolvedEntry; data: unknown };
export type DecodedCtor = { kind: 'ctor-call'; entry: ResolvedEntry; args: Record<string, unknown> };
