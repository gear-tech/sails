/**
 * @jest-environment jsdom
 */

import { ApiProvider } from '@gear-js/react-hooks';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook as renderReactHook, waitFor } from '@testing-library/react';
import { ReactNode } from 'react';

import { Program } from './demo/lib';
import { useProgram } from './demo/hooks';

const API_ARGS = { endpoint: 'ws://127.0.0.1:9944' };
const QUERY_CLIENT = new QueryClient();

const Providers = ({ children }: { children: ReactNode }) => (
  <ApiProvider initialArgs={API_ARGS}>
    <QueryClientProvider client={QUERY_CLIENT}>{children}</QueryClientProvider>
  </ApiProvider>
);

const renderHook = <TProps, TReturn>(hook: (initialProps: TProps) => TReturn) =>
  renderReactHook(hook, { wrapper: Providers });

describe('program hook', () => {
  test('useProgram hook', async () => {
    const { result } = renderHook(() => useProgram({ id: '0x01' }));

    await waitFor(() => {
      expect(result.current.data).toBeInstanceOf(Program);
      expect(result.current.data.programId).toBe('0x01');
    });
  });
});
