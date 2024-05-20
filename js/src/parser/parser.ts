import wasmParserBytes from './wasm-bytes.js';
import {
  DefKind,
  EnumDef,
  EnumVariant,
  FixedSizeArrayDef,
  MapDef,
  OptionalDef,
  PrimitiveDef,
  ResultDef,
  StructDef,
  StructField,
  Type,
  TypeDef,
  UserDefinedDef,
  VecDef,
} from './types.js';
import { Ctor, CtorFunc, Program } from './program.js';
import { FuncParam, Service, ServiceEvent, ServiceFunc } from './service.js';

const WASM_PAGE_SIZE = 0x10000;

interface ParserInstance extends WebAssembly.Instance {
  exports: {
    parse_idl: (idl_ptr: number, idl_len: number) => number;
    free_program: (program_ptr: number) => void;
    accept_program: (program_ptr: number, ctx: number) => void;
    accept_ctor: (ctor_ptr: number, ctx: number) => void;
    accept_ctor_func: (func_ptr: number, ctx: number) => void;
    accept_service: (service_ptr: number, ctx: number) => void;
    accept_service_func: (func_ptr: number, ctx: number) => void;
    accept_func_param: (func_param_ptr: number, ctx: number) => void;
    accept_type: (type_ptr: number, ctx: number) => void;
    accept_type_decl: (type_decl_ptr: number, ctx: number) => void;
    accept_struct_def: (struct_def_ptr: number, ctx: number) => void;
    accept_struct_field: (struct_field_ptr: number, ctx: number) => void;
    accept_enum_def: (enum_def_ptr: number, ctx: number) => void;
    accept_enum_variant: (enum_variant_ptr: number, ctx: number) => void;
    accept_service_event: (event_ptr: number, ctx: number) => void;
  };
}

export class WasmParser {
  private _memory: WebAssembly.Memory;
  private _instance: ParserInstance;
  private _exports: ParserInstance['exports'];
  private _encoder: TextEncoder;
  private _program: Program;
  private _memPtr: number;
  private _idlLen: number;
  private _numberOfGrownPages = 0;

  constructor() {
    this._encoder = new TextEncoder();
  }

  private async _decompressWasm() {
    const binaryStr = atob(wasmParserBytes);

    const binaryBase64 = new Uint8Array(binaryStr.length);

    for (let i = 0; i < binaryStr.length; i++) {
      binaryBase64[i] = binaryStr.charCodeAt(i);
    }

    const ds = new DecompressionStream('gzip');
    const decompressed = new Response(binaryBase64).body.pipeThrough<Uint8Array>(ds);

    const reader = decompressed.getReader();
    const bytes = [];

    while (true) {
      const { value, done } = await reader.read();

      if (done) break;

      bytes.push(...value);
    }

    return new Uint8Array(bytes).buffer;
  }

  async init() {
    const wasmBuf = await this._decompressWasm();

    const $ = this;

    $._memory = new WebAssembly.Memory({ initial: 17 });

    const source = await WebAssembly.instantiate(wasmBuf, {
      env: {
        memory: $._memory,
        visit_type: (_, type_ptr: number) => {
          const type = new Type(type_ptr, $._memory);
          const id = $._program.addType(type);
          $._instance.exports.accept_type(type_ptr, id);
        },
        visit_optional_type_decl: (ctx: number, optional_type_decl_ptr: number) => {
          const type = $._program.getContext(ctx);
          const def = new OptionalDef(optional_type_decl_ptr, $._memory);
          type.setDef(new TypeDef(def, DefKind.Optional));
          $._program.addContext(def.rawPtr, def);
          $._exports.accept_type_decl(optional_type_decl_ptr, def.rawPtr);
        },
        visit_vector_type_decl: (ctx: number, vector_type_decl_ptr: number) => {
          const type = $._program.getContext(ctx);
          const def = new VecDef(vector_type_decl_ptr, $._memory);
          type.setDef(new TypeDef(def, DefKind.Vec));
          $._program.addContext(def.rawPtr, def);
          $._exports.accept_type_decl(vector_type_decl_ptr, def.rawPtr);
        },
        visit_array_type_decl: (ctx: number, array_type_decl_ptr: number, len: number) => {
          const type = $._program.getContext(ctx);
          const def = new FixedSizeArrayDef(array_type_decl_ptr, len, $._memory);
          type.setDef(new TypeDef(def, DefKind.FixedSizeArray));
          $._program.addContext(def.rawPtr, def);
          $._exports.accept_type_decl(array_type_decl_ptr, def.rawPtr);
        },
        visit_map_type_decl: (ctx: number, key_type_decl_ptr: number, value_type_decl_ptr: number) => {
          const type = $._program.getContext(ctx);
          const def = new MapDef(key_type_decl_ptr, value_type_decl_ptr, $._memory);
          type.setDef(new TypeDef(def, DefKind.Map));
          $._program.addContext(def.key.rawPtr, def.key);
          $._program.addContext(def.value.rawPtr, def.value);

          $._exports.accept_type_decl(key_type_decl_ptr, def.key.rawPtr);
          $._exports.accept_type_decl(value_type_decl_ptr, def.value.rawPtr);
        },
        visit_result_type_decl: (ctx: number, ok_type_decl_ptr: number, err_type_decl_ptr: number) => {
          const type = $._program.getContext(ctx);
          const def = new ResultDef(ok_type_decl_ptr, err_type_decl_ptr, $._memory);
          type.setDef(new TypeDef(def, DefKind.Result));
          $._program.addContext(def.ok.rawPtr, def.ok);
          $._program.addContext(def.err.rawPtr, def.err);

          $._exports.accept_type_decl(ok_type_decl_ptr, def.ok.rawPtr);
          $._exports.accept_type_decl(err_type_decl_ptr, def.err.rawPtr);
        },
        visit_primitive_type_id: (ctx: number, primitive_type_id: number) => {
          const type = $._program.getContext(ctx);
          const def = new PrimitiveDef(primitive_type_id);
          type.setDef(new TypeDef(def, DefKind.Primitive));
        },
        visit_user_defined_type_id: (
          ctx: number,
          user_defined_type_id_ptr: number,
          user_defined_type_id_len: number,
        ) => {
          const type = $._program.getContext(ctx);
          const def = new UserDefinedDef(user_defined_type_id_ptr, user_defined_type_id_len, $._memory);
          type.setDef(new TypeDef(def, DefKind.UserDefined));
        },
        visit_struct_def: (ctx: number, struct_def_ptr: number) => {
          const type = $._program.getContext(ctx);
          const def = new StructDef(struct_def_ptr, $._memory);
          $._program.addContext(def.rawPtr, def);
          type.setDef(new TypeDef(def, DefKind.Struct));
          $._exports.accept_struct_def(struct_def_ptr, def.rawPtr);
        },
        visit_struct_field: (ctx: number, struct_field_ptr: number) => {
          const def = $._program.getContext(ctx);
          const field = new StructField(struct_field_ptr, $._memory);
          const id = def.addField(field);
          $._program.addContext(id, field);
          $._exports.accept_struct_field(struct_field_ptr, id);
        },
        visit_enum_def: (ctx: number, enum_def_ptr: number) => {
          const type = $._program.getType(ctx);
          const def = new EnumDef(enum_def_ptr, $._memory);
          $._program.addContext(def.rawPtr, def);
          type.setDef(new TypeDef(def, DefKind.Enum));
          $._exports.accept_enum_def(enum_def_ptr, def.rawPtr);
        },
        visit_enum_variant: (ctx: number, enum_variant_ptr: number) => {
          const def = $._program.getContext(ctx);
          const variant = new EnumVariant(enum_variant_ptr, $._memory);
          const id = def.addVariant(variant);
          $._program.addContext(id, variant);
          $._exports.accept_enum_variant(enum_variant_ptr, id);
        },
        visit_ctor: (_, ctor_ptr: number) => {
          $._program.addCtor(new Ctor(ctor_ptr, $._memory));
          $._exports.accept_ctor(ctor_ptr, 0);
        },
        visit_ctor_func: (_, func_ptr: number) => {
          const func = new CtorFunc(func_ptr, $._memory);
          $._program.ctor.addFunc(func);
          $._program.addContext(func.rawPtr, func);
          $._exports.accept_ctor_func(func_ptr, func.rawPtr);
        },
        visit_service: (_, service_ptr: number) => {
          const service = new Service(service_ptr, $._memory);
          $._program.addContext(service.rawPtr, service);
          $._program.addService(service);
          $._exports.accept_service(service_ptr, service.rawPtr);
        },
        visit_service_func: (ctx: number, func_ptr: number) => {
          const func = new ServiceFunc(func_ptr, $._memory);
          const service = $._program.getContext(ctx);
          service.addFunc(func);
          $._program.addContext(func.rawPtr, func);
          $._exports.accept_service_func(func_ptr, func.rawPtr);
        },
        visit_service_event: (ctx: number, event_ptr: number) => {
          const event = new ServiceEvent(event_ptr, $._memory);
          const service = $._program.getContext(ctx);
          service.addEvent(event);
          $._program.addContext(event.rawPtr, event);
          $._exports.accept_service_event(event_ptr, event.rawPtr);
        },
        visit_func_param: (ctx: number, func_param_ptr: number) => {
          const param = new FuncParam(func_param_ptr, $._memory);
          const func = $._program.getContext(ctx);
          func.addFuncParam(param.rawPtr, param);
          $._program.addContext(param.rawPtr, param);
          $._exports.accept_func_param(func_param_ptr, param.rawPtr);
        },
        visit_func_output: (ctx: number, func_output_ptr: number) => {
          $._exports.accept_type_decl(func_output_ptr, ctx);
        },
      },
    });

    $._instance = source.instance as ParserInstance;
    $._exports = $._instance.exports;

    return $;
  }

  static async new(): Promise<WasmParser> {
    const parser = new WasmParser();
    return parser.init();
  }

  private fillMemory(idl: string) {
    const buf = this._encoder.encode(idl);
    this._idlLen = buf.length;

    const numberOfPages = Math.round(buf.length / WASM_PAGE_SIZE) + 1;

    if (!this._memPtr || numberOfPages > this._numberOfGrownPages) {
      this._memPtr = this._memory.grow(numberOfPages - this._numberOfGrownPages) * WASM_PAGE_SIZE;
      this._numberOfGrownPages = numberOfPages;
    }

    for (let i = 0; i < buf.length; i++) {
      new Uint8Array(this._memory.buffer)[i + this._memPtr] = buf[i];
    }
  }

  private clearMemory() {
    for (let i = 0; i < this._numberOfGrownPages * WASM_PAGE_SIZE; i++) {
      new Uint8Array(this._memory.buffer)[i + this._memPtr] = 0;
    }
    this._idlLen = null;
  }

  public parse(idl: string): Program {
    this.fillMemory(idl);

    const programPtr = this._instance.exports.parse_idl(this._memPtr, this._idlLen);

    this._program = new Program();
    this._instance.exports.accept_program(programPtr, 0);

    this._exports.free_program(programPtr);
    this.clearMemory();
    return this._program;
  }
}
