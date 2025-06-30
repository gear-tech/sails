import { NonZeroU32, ActorId, H256 } from 'sails-js';

declare global {
  export type ReferenceCount = [number];

  export interface DoThatParam {
    p1: NonZeroU32;
    p2: ActorId;
    p3: ManyVariants;
  }

  export type ManyVariants = 
    | { One: null }
    | { Two: number }
    | { Three: number | string | bigint | null }
    | { Four: { a: number; b: number | null } }
    | { Five: [string, H256] }
    | { Six: [number] };

  export type ManyVariantsReply = "One" | "Two" | "Three" | "Four" | "Five" | "Six";

  export type TupleStruct = [boolean];
};