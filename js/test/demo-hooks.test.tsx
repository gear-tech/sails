import { HexString } from '@gear-js/api';
import * as gearHooks from '@gear-js/react-hooks';
import type { UseProgramParameters, UseProgramQueryParameters } from '@gear-js/react-hooks';
import type {
  GenericTransactionReturn,
  SignAndSendOptions,
  TransactionReturn,
} from '@gear-js/react-hooks/dist/esm/hooks/sails/types';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook as renderReactHook, waitFor } from '@testing-library/react';
import * as fs from 'fs';
import path from 'path';
import { ReactNode } from 'react';
import { test, expect, vi, MockInstance, expectTypeOf, describe } from 'vitest';
import { SailsIdlParser } from 'sails-js-parser';
import { toLowerCaseFirst } from 'sails-js-util';

import { ActorId, NonZeroU32, Sails } from '..';
import { DoThatParam, Program, ReferenceCount } from './demo/lib';
import {
  useCounterAddedEvent,
  useDogPositionQuery,
  useDogWalkedEvent,
  usePrepareCounterAddTransaction,
  usePrepareReferencesIncrTransaction,
  useSendDogWalkTransaction,
  useSendThisThatDoThatTransaction,
  useThisThatThatQuery,
} from './demo/hooks';

const { useProgram, ...demoHooks } = await import('./demo/hooks');
const { ApiProvider } = gearHooks;

const getSails = async () => {
  const idlPath = path.resolve(__dirname, '../../examples/demo/client/demo.idl');
  const idl = fs.readFileSync(idlPath, 'utf8');
  const parser = await SailsIdlParser.new();

  return new Sails(parser).parseIdl(idl);
};

const apiArgs = { endpoint: 'ws://127.0.0.1:9944' };
const { services } = await getSails();
const queryClient = new QueryClient();
const useProgramSpy = vi.spyOn(gearHooks, 'useProgram');
const useSendTransactionSpy = vi.spyOn(gearHooks, 'useSendProgramTransaction');
const usePrepareTransactionSpy = vi.spyOn(gearHooks, 'usePrepareProgramTransaction');
const useQuerySpy = vi.spyOn(gearHooks, 'useProgramQuery');
const useEventSpy = vi.spyOn(gearHooks, 'useProgramEvent');

const Providers = ({ children }: { children: ReactNode }) => (
  <ApiProvider initialArgs={apiArgs}>
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  </ApiProvider>
);

const renderHook = <TProps, TReturn>(hook: (initialProps: TProps) => TReturn) =>
  renderReactHook(hook, { wrapper: Providers });

const testHookParameters = (
  type: 'functions' | 'queries' | 'events',
  getName: (value: string) => string,
  spy: MockInstance,
  extraArgs = {},
  getFunctionName: (value: string) => string = (value) => value
) => {
  const { result } = renderHook(() => useProgram({ id: '0x01' }));
  const program = result.current.data;
  const args = { program, ...extraArgs };

  Object.entries(services).forEach(([serviceName, service]) =>
    Object.keys(service[type]).forEach((functionName) => {
      const hookName = getName(`${serviceName}${functionName}`);
      const useHook: unknown = demoHooks[hookName];

      if (typeof useHook !== 'function') throw new Error(`Hook ${hookName} not found`);

      renderHook(() => useHook(args));

      expect(spy).toHaveBeenCalledWith({
        ...args,
        serviceName: toLowerCaseFirst(serviceName),
        functionName: toLowerCaseFirst(getFunctionName(functionName)),
      });
    })
  );
};

describe('useProgram', () => {
  test('parameters forwarding', () => {
    const args = { id: '0x01' as HexString, query: { enabled: true } };
    renderHook(() => useProgram(args));

    expect(useProgramSpy).toHaveBeenCalledWith({ library: Program, ...args });
  });

  test('program instance return', async () => {
    const { result } = renderHook(() => useProgram({ id: '0x01' }));

    await waitFor(() => expect(result.current.data).toBeInstanceOf(Program));
  });

  test('parameters type', () => {
    expectTypeOf(useProgram)
      .parameter(0)
      .toEqualTypeOf<{ id: HexString; query?: UseProgramParameters<Program>['query'] }>();
  });
});

describe('useSendTransaction', () => {
  test('parameters forwarding', () => {
    testHookParameters('functions', (name) => `useSend${name}Transaction`, useSendTransactionSpy);
  });

  test('parameters type', () => {
    expectTypeOf(useSendDogWalkTransaction).parameter(0).toEqualTypeOf<{ program: Program | undefined }>();
    expectTypeOf(useSendThisThatDoThatTransaction).parameter(0).toEqualTypeOf<{ program: Program | undefined }>();
  });

  test('mutation args type', () => {
    const { result: programResult } = renderHook(() => useProgram({ id: '0x01' }));
    const program = programResult.current.data;

    const { result } = renderHook(() => useSendDogWalkTransaction({ program }));
    const { sendTransaction } = result.current;

    const { result: anotherResult } = renderHook(() => useSendThisThatDoThatTransaction({ program }));
    const { sendTransaction: anotherSendTransaction } = anotherResult.current;

    expectTypeOf(sendTransaction)
      .parameter(0)
      .extract<SignAndSendOptions<unknown>>()
      .toMatchTypeOf<{ args: [number, number] }>();

    expectTypeOf(anotherSendTransaction)
      .parameter(0)
      .extract<SignAndSendOptions<unknown>>()
      .toMatchTypeOf<{ args: [DoThatParam] }>();
  });

  test('mutation return type', () => {
    const { result: programResult } = renderHook(() => useProgram({ id: '0x01' }));
    const program = programResult.current.data;

    const { result } = renderHook(() => useSendDogWalkTransaction({ program: programResult.current.data }));
    const { sendTransactionAsync } = result.current;

    const { result: anotherResult } = renderHook(() => useSendThisThatDoThatTransaction({ program }));
    const { sendTransactionAsync: anotherSendTransactionAsync } = anotherResult.current;

    expectTypeOf(sendTransactionAsync)
      .returns.resolves.pick('awaited')
      .toMatchTypeOf<{ awaited: { response: null } }>();

    expectTypeOf(anotherSendTransactionAsync)
      .returns.resolves.pick('awaited')
      .toMatchTypeOf<{ awaited: { response: { ok: [ActorId, NonZeroU32] } | { err: [string] } } }>();
  });
});

describe('usePrepareTransaction', () => {
  test('parameters forwarding', () => {
    testHookParameters('functions', (name) => `usePrepare${name}Transaction`, usePrepareTransactionSpy);
  });

  test('parameters type', () => {
    expectTypeOf(usePrepareCounterAddTransaction).parameter(0).toEqualTypeOf<{ program: Program | undefined }>();
    expectTypeOf(usePrepareReferencesIncrTransaction).parameter(0).toEqualTypeOf<{ program: Program | undefined }>();
  });

  test('mutation args type', () => {
    const { result: programResult } = renderHook(() => useProgram({ id: '0x01' }));
    const program = programResult.current.data;

    const { result } = renderHook(() => usePrepareCounterAddTransaction({ program }));
    const { prepareTransaction } = result.current;

    const { result: anotherResult } = renderHook(() => usePrepareReferencesIncrTransaction({ program }));
    const { prepareTransaction: anotherPrepareTransaction } = anotherResult.current;

    expectTypeOf(prepareTransaction).parameter(0).toMatchTypeOf<{ args: [number] }>();
    expectTypeOf(anotherPrepareTransaction).parameter(0).toMatchTypeOf<{ args: [] }>();
  });

  test('mutation return type', async () => {
    const { result: programResult } = renderHook(() => useProgram({ id: '0x01' }));
    const program = programResult.current.data;

    const { result } = renderHook(() => usePrepareCounterAddTransaction({ program }));
    const { prepareTransactionAsync } = result.current;

    const { result: anotherResult } = renderHook(() => usePrepareReferencesIncrTransaction({ program }));
    const { prepareTransactionAsync: anotherPrepareTransactionAsync } = anotherResult.current;

    expectTypeOf(prepareTransactionAsync)
      .returns.resolves.pick('transaction')
      .toEqualTypeOf<{ transaction: TransactionReturn<(value: number) => GenericTransactionReturn<number>> }>();

    expectTypeOf(anotherPrepareTransactionAsync)
      .returns.resolves.pick('transaction')
      .toEqualTypeOf<{ transaction: TransactionReturn<() => GenericTransactionReturn<ReferenceCount>> }>();
  });
});

describe('useQuery', () => {
  test('parameters forwarding', () => {
    testHookParameters('queries', (name) => `use${name}Query`, useQuerySpy, { query: { enabled: true } });
  });

  test('parameters type', () => {
    expectTypeOf(useDogPositionQuery)
      .parameter(0)
      .toEqualTypeOf<
        Omit<
          UseProgramQueryParameters<
            Program,
            'dog',
            'position',
            [originAddress?: string, value?: string | number | bigint, atBlock?: HexString],
            [number, number]
          >,
          'serviceName' | 'functionName'
        >
      >();

    expectTypeOf(useThisThatThatQuery)
      .parameter(0)
      .toEqualTypeOf<
        Omit<
          UseProgramQueryParameters<
            Program,
            'thisThat',
            'that',
            [originAddress?: string, value?: string | number | bigint, atBlock?: HexString],
            { ok: string } | { err: string }
          >,
          'serviceName' | 'functionName'
        >
      >();
  });

  test('query return type', () => {
    const { result: programResult } = renderHook(() => useProgram({ id: '0x01' }));
    const program = programResult.current.data;

    const { result } = renderHook(() => useDogPositionQuery({ program, args: [] }));
    const { data } = result.current;

    const { result: anotherResult } = renderHook(() => useThisThatThatQuery({ program, args: [] }));
    const { data: anotherData } = anotherResult.current;

    expectTypeOf(data).toEqualTypeOf<[number, number]>();
    expectTypeOf(anotherData).toEqualTypeOf<{ ok: string } | { err: string }>();
  });
});

describe('useEvent', () => {
  test('parameters forwarding', () => {
    testHookParameters(
      'events',
      (name) => `use${name}Event`,
      useEventSpy,
      { query: { enabled: true } },
      (name) => `subscribeTo${name}Event`
    );
  });

  test('parameters + onData args type', () => {
    expectTypeOf(useCounterAddedEvent)
      .parameter(0)
      .toEqualTypeOf<{ program: Program | undefined; onData: (value: number) => void }>();

    expectTypeOf(useDogWalkedEvent).parameter(0).toEqualTypeOf<{
      program: Program | undefined;
      onData: (value: { from: [number, number]; to: [number, number] }) => void;
    }>();
  });
});
