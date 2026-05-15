import { GearCoreErrorsSimpleReplyCode, ReplyCode } from '@gear-js/api';
import { TypeRegistry } from '@polkadot/types';
import { compactAddLength, stringToU8a } from '@polkadot/util';

export function throwOnErrorReply(
  code: GearCoreErrorsSimpleReplyCode,
  payload: Uint8Array,
  specVersion: number,
  registry: TypeRegistry,
) {
  if (code.isSuccess) return;

  const replyCode = new ReplyCode(code.toU8a(), specVersion);

  if (!replyCode.isError) {
    throw new Error(`Unable to decode reply code. ${code.toU8a()}`);
  }

  const reason = replyCode.errorReason;

  if (reason.isExecution) {
    const error = reason.executionReason.isUserspacePanic
      ? new Error(registry.createType('String', payload).toString())
      : new Error(reason.executionReason.explanation);
    throw error;
  } else if (reason.isUnavailableActor) {
    const error = reason.unavailableActorReason.isProgramExited
      ? new Error(`Program exited. Program inheritor is ${registry.createType('[u8;32]', payload).toHex()}`)
      : new Error(reason.unavailableActorReason.explanation);
    throw error;
  } else {
    throw new Error(reason.explanation);
  }
}

export function stringToU8aWithPrefix(value: string): Uint8Array {
  const str = stringToU8a(value);
  return compactAddLength(str);
}
