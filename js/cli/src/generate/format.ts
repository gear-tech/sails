export function formatDocs(docs: string): string[] {
  if (!docs) return []
  const result = new Array<string>()

  const lines = docs.split("\n")
  for (const line of lines) {
    result.push(` * ${line.trim()}`)
  }

  return ["/**", ...result, "*/"]
}
