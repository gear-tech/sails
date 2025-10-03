import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { confirm } from '@inquirer/prompts';
import { Sails } from 'sails-js';
import path from 'node:path';
import { ServiceGenerator } from './service-gen.js';
import { TypesGenerator } from './types-gen.js';
import { Output } from './output.js';
import * as config from '../config.json';

export class ProjectBuilder {
  private projectPath = ['.', 'src'];
  private isProject: boolean = true;
  private isAutomaricOverride: boolean = false;
  private isEmbeddedTypes: boolean = false;

  constructor(
    private sails: Sails,
    private name: string,
  ) {}

  private async canCreateFile(filePath: string): Promise<boolean> {
    if (this.isAutomaricOverride || !existsSync(filePath)) {
      return true;
    }

    const answer = await confirm({
      message: `File ${filePath} exists. Do you want to overwrite?`,
      default: false,
    });

    return answer;
  }

  public generateLib(): string {
    const out = new Output();

    const serviceGen = new ServiceGenerator(out, this.sails.program, this.sails.scaleCodecTypes);
    serviceGen.generate(this.name);

    return out.finalize();
  }

  public generateTypes(): string | null {
    const out = new Output();

    const typesGen = new TypesGenerator(out, this.sails.program);
    typesGen.generate();

    return out.finalize();
  }

  setIdlPath(path: string) {
    const idl = readFileSync(path, 'utf8');
    this.sails.parseIdl(idl);

    return this;
  }

  setRootPath(path: string) {
    this.projectPath[0] = path ?? '.';

    return this;
  }

  setIsProject(isProject: boolean) {
    this.isProject = isProject;

    return this;
  }

  setAutomaticOverride(isAutomaricOverride: boolean) {
    this.isAutomaricOverride = isAutomaricOverride;

    return this;
  }

  setIsEmbeddedTypes(isEmbeddedTypes: boolean) {
    this.isEmbeddedTypes = isEmbeddedTypes;

    return this;
  }

  async build() {
    const rootPath = this.projectPath[0];
    const srcPath = path.join(...this.projectPath);

    const libPath = this.isProject ? srcPath : rootPath;

    if (!existsSync(libPath)) {
      mkdirSync(libPath, { recursive: true });
    }

    const libCode = this.generateLib();
    const libFile = path.join(libPath, 'lib.ts');
    if (await this.canCreateFile(libFile)) {
      writeFileSync(libFile, libCode);
    } else {
      throw new Error(`Failed to write file ${libFile}`);
    }

    const typesCode = this.generateTypes();
    const typesFile = path.join(libPath, 'global.d.ts');
    if (typesCode !== null) {
      if (await this.canCreateFile(typesFile)) {
        writeFileSync(typesFile, typesCode);
      } else {
        throw new Error(`Failed to write file ${typesFile}`);
      }
    }

    if (!this.isProject) {
      console.log(`Lib generated at ${libPath}`);
      return;
    }

    const tsconfigPath = path.join(rootPath, 'tsconfig.json');
    const pkgJsonPath = path.join(rootPath, 'package.json');

    if (await this.canCreateFile(tsconfigPath)) {
      writeFileSync(tsconfigPath, JSON.stringify(config.tsconfig, null, 2));
    }

    if (await this.canCreateFile(pkgJsonPath)) {
      writeFileSync(
        pkgJsonPath,
        JSON.stringify(
          {
            name: this.name,
            type: 'module',
            dependencies: {
              '@gear-js/api': config.versions['gear-js'],
              '@polkadot/api': config.versions['polkadot-api'],
              'sails-js': config.versions['sails-js'],
            },
            devDependencies: {
              typescript: config.versions['typescript'],
            },
            scripts: {
              build: 'tsc',
            },
          },
          null,
          2,
        ),
      );
    }

    console.log(`Lib generated at ${srcPath}`);
  }
}
