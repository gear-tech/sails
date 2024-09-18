import { HexString } from '@gear-js/api';
import * as GearHooks from '@gear-js/react-hooks';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook as renderReactHook } from '@testing-library/react';
import { ReactNode } from 'react';
import { test, expect, vi } from 'vitest';

import { Program } from './demo/lib';
import {
  useCounterAddedEvent,
  useCounterValueQuery,
  useDogAvgWeightQuery,
  useDogBarkedEvent,
  usePrepareCounterAddTransaction,
  usePrepareDogMakeSoundTransaction,
  usePreparePingPongPingTransaction,
  usePrepareReferencesSetNumTransaction,
  usePrepareThisThatNoopTransaction,
  useProgram,
  useReferencesLastByteQuery,
  useSendCounterAddTransaction,
  useSendDogMakeSoundTransaction,
  useSendPingPongPingTransaction,
  useSendReferencesSetNumTransaction,
  useSendThisThatNoopTransaction,
  useThisThatThatQuery,
} from './demo/hooks';

const QUERY_CLIENT = new QueryClient();

const useProgramSpy = vi.spyOn(GearHooks, 'useProgram');
const useSendTransactionSpy = vi.spyOn(GearHooks, 'useSendProgramTransaction');
const usePrepareTransactionSpy = vi.spyOn(GearHooks, 'usePrepareProgramTransaction');
const useQuerySpy = vi.spyOn(GearHooks, 'useProgramQuery');
const useEventSpy = vi.spyOn(GearHooks, 'useProgramEvent');

const Providers = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider client={QUERY_CLIENT}>{children}</QueryClientProvider>
);

const renderHook = <TProps, TReturn>(hook: (initialProps: TProps) => TReturn) =>
  renderReactHook(hook, { wrapper: Providers });

test('useProgram', () => {
  const ARGS = { id: '0x01' as HexString, query: { enabled: true } };
  renderHook(() => useProgram(ARGS));

  expect(useProgramSpy).toHaveBeenCalledWith({ library: Program, ...ARGS });
});

test('useSendTransaction', () => {
  const { result } = renderHook(() => useProgram({ id: '0x01' }));
  const program = result.current.data;

  renderHook(() => useSendCounterAddTransaction({ program }));
  renderHook(() => useSendDogMakeSoundTransaction({ program }));
  renderHook(() => useSendPingPongPingTransaction({ program }));
  renderHook(() => useSendReferencesSetNumTransaction({ program }));
  renderHook(() => useSendThisThatNoopTransaction({ program }));

  expect(useSendTransactionSpy).toHaveBeenCalledWith({ program, serviceName: 'counter', functionName: 'add' });
  expect(useSendTransactionSpy).toHaveBeenCalledWith({ program, serviceName: 'dog', functionName: 'makeSound' });
  expect(useSendTransactionSpy).toHaveBeenCalledWith({ program, serviceName: 'pingPong', functionName: 'ping' });
  expect(useSendTransactionSpy).toHaveBeenCalledWith({ program, serviceName: 'references', functionName: 'setNum' });
  expect(useSendTransactionSpy).toHaveBeenCalledWith({ program, serviceName: 'thisThat', functionName: 'noop' });
});

test('usePrepareTransaction', () => {
  const { result } = renderHook(() => useProgram({ id: '0x01' }));
  const program = result.current.data;

  renderHook(() => usePrepareCounterAddTransaction({ program }));
  renderHook(() => usePrepareDogMakeSoundTransaction({ program }));
  renderHook(() => usePreparePingPongPingTransaction({ program }));
  renderHook(() => usePrepareReferencesSetNumTransaction({ program }));
  renderHook(() => usePrepareThisThatNoopTransaction({ program }));

  expect(usePrepareTransactionSpy).toHaveBeenCalledWith({ program, serviceName: 'counter', functionName: 'add' });
  expect(usePrepareTransactionSpy).toHaveBeenCalledWith({ program, serviceName: 'dog', functionName: 'makeSound' });
  expect(usePrepareTransactionSpy).toHaveBeenCalledWith({ program, serviceName: 'pingPong', functionName: 'ping' });
  expect(usePrepareTransactionSpy).toHaveBeenCalledWith({ program, serviceName: 'references', functionName: 'setNum' });
  expect(usePrepareTransactionSpy).toHaveBeenCalledWith({ program, serviceName: 'thisThat', functionName: 'noop' });
});

test('useQuery', () => {
  const { result } = renderHook(() => useProgram({ id: '0x01' }));
  const ARGS = { program: result.current.data, query: { enabled: true } };

  renderHook(() => useCounterValueQuery({ ...ARGS, args: [] }));
  renderHook(() => useDogAvgWeightQuery({ ...ARGS, args: [] }));
  renderHook(() => useReferencesLastByteQuery({ ...ARGS, args: [] }));
  renderHook(() => useThisThatThatQuery({ ...ARGS, args: [] }));

  expect(useQuerySpy).toHaveBeenCalledWith({ ...ARGS, serviceName: 'counter', functionName: 'value', args: [] });
  expect(useQuerySpy).toHaveBeenCalledWith({ ...ARGS, serviceName: 'dog', functionName: 'avgWeight', args: [] });
  expect(useQuerySpy).toHaveBeenCalledWith({ ...ARGS, serviceName: 'references', functionName: 'lastByte', args: [] });
  expect(useQuerySpy).toHaveBeenCalledWith({ ...ARGS, serviceName: 'thisThat', functionName: 'that', args: [] });
});

test('useEvent', () => {
  const { result } = renderHook(() => useProgram({ id: '0x01' }));
  const ARGS = { program: result.current.data, query: { enabled: true }, onData: () => {} };

  renderHook(() => useCounterAddedEvent(ARGS));
  renderHook(() => useDogBarkedEvent(ARGS));

  expect(useEventSpy).toHaveBeenCalledWith({ ...ARGS, serviceName: 'counter', functionName: 'subscribeToAddedEvent' });
  expect(useEventSpy).toHaveBeenCalledWith({ ...ARGS, serviceName: 'dog', functionName: 'subscribeToBarkedEvent' });
});
