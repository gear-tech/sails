export function formatDocs(docs: string): string[] {
  if (!docs) return [];
  const result = new Array<string>();

  const lines = docs.split('\n');
  for (const line of lines) {
    // Escape comment terminators so doc text cannot break out of the
    // generated JSDoc block and inject executable TypeScript.
    result.push(` * ${line.trim().replaceAll('*/', String.raw`*\/`)}`);
  }

  return ['/**', ...result, '*/'];
}
