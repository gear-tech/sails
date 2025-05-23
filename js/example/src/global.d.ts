import { NonZeroU32, ActorId, H256 } from 'sails-js';

declare global {
  export type ReferenceCount = [number];

  export interface DoThatParam {
    p1: NonZeroU32;
    p2: ActorId;
    p3: ManyVariants;
  }

  export type ManyVariants = 
    | { one: null }
    | { two: number }
    | { three: number | string | bigint | null }
    | { four: { a: number; b: number | null } }
    | { five: [string, H256] }
    | { six: [number] };

  export type TupleStruct = [boolean];
};