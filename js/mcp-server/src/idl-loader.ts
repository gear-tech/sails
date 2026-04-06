import { readFile } from 'node:fs/promises';
import path from 'node:path';

interface IdlSource {
  content: string;
  id: string;
}

interface IdlLoader {
  load(path: string): Promise<IdlSource>;
  resolve(basePath: string, includePath: string): string | null;
}

/**
 * Loads IDL files from the local filesystem.
 * Resolves relative include paths against the directory of the including file.
 */
class FsLoader implements IdlLoader {
  async load(filePath: string): Promise<IdlSource> {
    const canonical = path.resolve(filePath);
    const content = await readFile(canonical, 'utf8');
    return { content, id: canonical };
  }

  resolve(basePath: string, includePath: string): string | null {
    if (includePath.startsWith('git://')) return null;
    const baseDir = path.dirname(path.resolve(basePath));
    return path.resolve(baseDir, includePath);
  }
}

/**
 * Loads IDL files from git:// URLs via HTTPS.
 * Format: git://github.com/user/repo/path/to/file.idl?branch
 */
class GitLoader implements IdlLoader {
  async load(path: string): Promise<IdlSource> {
    const url = this.toRawUrl(path);
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch ${url}: ${response.status} ${response.statusText}`);
    }
    const content = await response.text();
    return { content, id: path };
  }

  resolve(basePath: string, includePath: string): string | null {
    if (includePath.startsWith('git://')) return includePath;
    if (!basePath.startsWith('git://')) return null;

    // Resolve relative paths within the same git repo
    const baseUrl = new URL(basePath.replace('git://', 'https://'));
    const baseParts = baseUrl.pathname.split('/');
    baseParts.pop(); // remove filename
    baseParts.push(includePath);
    baseUrl.pathname = baseParts.join('/');
    return 'git://' + baseUrl.hostname + baseUrl.pathname + baseUrl.search;
  }

  private toRawUrl(gitUrl: string): string {
    // git://github.com/user/repo/path/to/file.idl?branch
    const url = new URL(gitUrl.replace('git://', 'https://'));
    const parts = url.pathname.split('/').filter(Boolean);
    if (parts.length < 3) {
      throw new Error(`Invalid git URL: ${gitUrl}. Expected git://host/user/repo/path?branch`);
    }
    const [user, repo, ...pathParts] = parts;
    const branch = (url.searchParams.get('branch') ?? url.search.slice(1)) || 'main';
    const host = url.hostname;

    if (host === 'github.com') {
      return `https://raw.githubusercontent.com/${user}/${repo}/${branch}/${pathParts.join('/')}`;
    }
    // Fallback: assume GitLab-style raw URL
    return `https://${host}/${user}/${repo}/-/raw/${branch}/${pathParts.join('/')}`;
  }
}

const fsLoader = new FsLoader();
const gitLoader = new GitLoader();
const defaultLoaders: IdlLoader[] = [fsLoader, gitLoader];

/**
 * Preprocess an IDL file, resolving !@include directives recursively.
 * Mirrors the logic in rs/idl-parser-v2/src/preprocess/mod.rs.
 */
export async function preprocessIdl(
  path: string,
  loaders: IdlLoader[] = defaultLoaders,
): Promise<string> {
  const visited = new Set<string>();
  const result: string[] = [];
  await preprocessRecursive(path, loaders, visited, result);
  return result.join('');
}

async function preprocessRecursive(
  path: string,
  loaders: IdlLoader[],
  visited: Set<string>,
  out: string[],
): Promise<void> {
  // Find a loader that can handle this path
  const loader = loaders.find((l) => l.resolve(path, path) !== null);
  if (!loader) {
    throw new Error(`No loader can handle path: ${path}`);
  }

  const source = await loader.load(path);

  // Deduplication
  if (visited.has(source.id)) {
    return;
  }
  visited.add(source.id);

  for (const line of source.content.split('\n')) {
    const trimmed = line.trim();

    if (trimmed.startsWith('!@include:')) {
      const rest = trimmed.slice('!@include:'.length).trim();
      const includePath = rest.replaceAll(/^["']|["']$/g, '');

      if (!includePath) {
        throw new Error('Invalid include directive');
      }

      // Find a loader that can resolve this include
      let nextPath: string | null = null;
      for (const l of loaders) {
        nextPath = l.resolve(path, includePath);
        if (nextPath !== null) break;
      }

      if (nextPath === null) {
        throw new Error(`No loader can resolve include '${includePath}' from: ${path}`);
      }

      await preprocessRecursive(nextPath, loaders, visited, out);

      // Ensure newline after included content
      if (out.length > 0 && !out.at(-1).endsWith('\n')) {
        out.push('\n');
      }
    } else {
      out.push(line + '\n');
    }
  }
}
