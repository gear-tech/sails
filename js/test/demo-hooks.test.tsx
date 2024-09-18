import { HexString } from '@gear-js/api';
import * as GearHooks from '@gear-js/react-hooks';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook as renderReactHook } from '@testing-library/react';
import * as fs from 'fs';
import path from 'path';
import { ReactNode } from 'react';
import { test, expect, vi, MockInstance } from 'vitest';
import { SailsIdlParser } from 'sails-js-parser';
import { toLowerCaseFirst } from 'sails-js-util';

import { Sails } from '..';
import { Program } from './demo/lib';

const { useProgram, ...demoHooks } = await import('./demo/hooks');

const getSails = async () => {
  const idlPath = path.resolve(__dirname, '../../examples/demo/client/demo.idl');
  const idl = fs.readFileSync(idlPath, 'utf8');
  const parser = await SailsIdlParser.new();

  return new Sails(parser).parseIdl(idl);
};

const SAILS = await getSails();
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

const testHookParameters = (
  type: 'functions' | 'queries' | 'events',
  getName: (value: string) => string,
  spy: MockInstance,
  extraArgs = {},
  getFunctionName?: (value: string) => string,
) => {
  const { result } = renderHook(() => useProgram({ id: '0x01' }));
  const program = result.current.data;

  Object.entries(SAILS.services).forEach(([serviceName, { [type]: functions }]) => {
    Object.keys(functions).forEach((functionName) => {
      const hookName = getName(`${serviceName}${functionName}`);
      const useHook: unknown = demoHooks[hookName];

      if (typeof useHook !== 'function') throw new Error(`Hook ${hookName} not found`);

      renderHook(() => useHook({ program, ...extraArgs }));

      expect(spy).toHaveBeenCalledWith({
        program,
        serviceName: toLowerCaseFirst(serviceName),
        functionName: toLowerCaseFirst(getFunctionName ? getFunctionName(functionName) : functionName),
        ...extraArgs,
      });
    });
  });
};

test('useProgram parameters forwarding', () => {
  const ARGS = { id: '0x01' as HexString, query: { enabled: true } };
  renderHook(() => useProgram(ARGS));

  expect(useProgramSpy).toHaveBeenCalledWith({ library: Program, ...ARGS });
});

test('useSendTransaction parameters forwarding', () => {
  testHookParameters('functions', (name) => `useSend${name}Transaction`, useSendTransactionSpy);
});

test('usePrepareTransaction parameters forwarding', () => {
  testHookParameters('functions', (name) => `usePrepare${name}Transaction`, usePrepareTransactionSpy);
});

test('useQuery parameters forwarding', () => {
  testHookParameters('queries', (name) => `use${name}Query`, useQuerySpy, { query: { enabled: true } });
});

test('useEvent parameters forwarding', () => {
  testHookParameters(
    'events',
    (name) => `use${name}Event`,
    useEventSpy,
    { query: { enabled: true }, onData: () => {} },
    (name) => `subscribeTo${name}Event`,
  );
});
