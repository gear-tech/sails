describe('parser-v2 success', () => {
  test('parses demo.idl', async () => {
    const { SailsIdlParser } = await import('../src/parser.js');
    const parser = new SailsIdlParser();
    await parser.init();

    const idl = await import('../test/fixture/demo.js')
    const doc = parser.parse(idl.default);

    expect(doc.program?.name).toBe('DemoClient');
    expect(doc.services?.map((service) => service.name)).toEqual([
      'PingPong',
      'Counter',
      'MammalService',
      'WalkerService',
      'Dog',
      'References',
      'ThisThat',
      'ValueFee',
      'Chaos'
    ]);
  });
});
