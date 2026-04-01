import { SailsProgram } from 'sails-js';
import type { IIdlDoc } from 'sails-js-types';

export interface RegisteredProgram {
  name: string;
  doc: IIdlDoc;
  program: SailsProgram;
}

/**
 * Session-scoped registry of parsed Sails programs.
 * Keyed by program name (from IDL or user-provided).
 */
class ProgramRegistry {
  private programs = new Map<string, RegisteredProgram>();

  register(name: string, doc: IIdlDoc): RegisteredProgram {
    const program = new SailsProgram(doc);
    const entry: RegisteredProgram = { name, doc, program };
    this.programs.set(name, entry);
    return entry;
  }

  get(name: string): RegisteredProgram | undefined {
    return this.programs.get(name);
  }

  getOrThrow(name: string): RegisteredProgram {
    const entry = this.programs.get(name);
    if (!entry) {
      const available = this.list().map((p) => p.name);
      throw new Error(
        `Program "${name}" not found. Available: [${available.join(', ')}]. Use sails_parse_idl or sails_load_idl first.`,
      );
    }
    return entry;
  }

  list(): RegisteredProgram[] {
    return [...this.programs.values()];
  }

  has(name: string): boolean {
    return this.programs.has(name);
  }
}

export const registry = new ProgramRegistry();
