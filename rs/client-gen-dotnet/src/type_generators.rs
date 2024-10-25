use crate::helpers::*;
use convert_case::{Case, Casing};
use csharp::{block_comment, Tokens};
use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) fn generate_type_decl_code<'a>(type_decl: &TypeDecl) -> String {
    let mut type_decl_generator = TypeDeclGenerator {
        code: String::new(),
        generated_types: &Vec::new(),
    };
    visitor::accept_type_decl(type_decl, &mut type_decl_generator);
    type_decl_generator.code
}

pub(crate) fn generate_type_decl_code_with_enums<'a>(
    type_decl: &TypeDecl,
    generated_types: &'a Vec<&'a Type>,
) -> String {
    let mut type_decl_generator = TypeDeclGenerator {
        code: String::new(),
        generated_types,
    };
    visitor::accept_type_decl(type_decl, &mut type_decl_generator);
    type_decl_generator.code
}

pub(crate) fn generate_struct_def_code_with_enums<'a>(
    struct_def: &StructDef,
    generated_types: &'a Vec<&'a Type>,
) -> String {
    let mut type_decl_generator = TypeDeclGenerator {
        code: String::new(),
        generated_types,
    };
    visitor::accept_struct_def(struct_def, &mut type_decl_generator);
    type_decl_generator.code
}

pub(crate) fn generate_type_decl_with_path(type_decl: &TypeDecl, path: String) -> String {
    let mut type_decl_generator = TypeDeclGenerator {
        code: String::new(),
        generated_types: &Vec::new(),
    };
    visitor::accept_type_decl(type_decl, &mut type_decl_generator);
    type_decl_generator.code
}

pub(crate) struct TopLevelTypeGenerator<'a> {
    type_name: &'a str,
    generated_types: &'a Vec<&'a Type>,
    tokens: Tokens,
}

impl<'a> TopLevelTypeGenerator<'a> {
    pub(crate) fn new(type_name: &'a str, generated_types: &'a Vec<&'a Type>) -> Self {
        Self {
            type_name,
            generated_types,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        self.tokens
    }
}

impl<'a> Visitor<'a> for TopLevelTypeGenerator<'a> {
    fn visit_type(&mut self, type_: &'a Type) {
        self.tokens.push();
        self.tokens.append(summary_comment(type_.docs()));
        self.tokens.push();
        visitor::accept_type(type_, self);
        self.tokens.line();
    }

    fn visit_struct_def(&mut self, struct_def: &'a StructDef) {
        let mut struct_def_generator =
            StructDefGenerator::new(self.type_name, self.generated_types);
        struct_def_generator.visit_struct_def(struct_def);
        self.tokens.extend(struct_def_generator.finalize());
    }

    fn visit_enum_def(&mut self, enum_def: &'a EnumDef) {
        let mut enum_def_generator = EnumDefGenerator::new(self.type_name, self.generated_types);
        enum_def_generator.visit_enum_def(enum_def);
        self.tokens.extend(enum_def_generator.finalize());
    }
}

struct StructDefGenerator<'a> {
    type_name: &'a str,
    generated_types: &'a Vec<&'a Type>,
    is_tuple_struct: bool,
    props_tokens: Tokens,
    encode_tokens: Tokens,
    decode_tokens: Tokens,
}

impl<'a> StructDefGenerator<'a> {
    fn new(type_name: &'a str, generated_types: &'a Vec<&'a Type>) -> Self {
        Self {
            type_name,
            generated_types,
            is_tuple_struct: false,
            props_tokens: Tokens::new(),
            encode_tokens: Tokens::new(),
            decode_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        quote! {
            [global::Substrate.NetApi.Attributes.SubstrateNodeType(global::Substrate.NetApi.Model.Types.Metadata.Base.TypeDefEnum.Composite)]
            public sealed partial class $(self.type_name) : global::Substrate.NetApi.Model.Types.Base.BaseType
            {
                $(self.props_tokens)

                $(block_comment(vec!["<inheritdoc/>"]))
                public override string TypeName() => $(quoted(self.type_name));

                $(block_comment(vec!["<inheritdoc/>"]))
                public override byte[] Encode()
                {
                    var result = new List<byte>();
                    $(self.encode_tokens)
                    return result.ToArray();
                }

                $(block_comment(vec!["<inheritdoc/>"]))
                public override void Decode(byte[] byteArray, ref int p)
                {
                    var start = p;
                    $(self.decode_tokens)
                    var bytesLength = p - start;
                    this.TypeSize = bytesLength;
                    this.Bytes = new byte[bytesLength];
                    global::System.Array.Copy(byteArray, start, this.Bytes, 0, bytesLength);
                }
            }
        }
    }

    fn tuple_struct(&mut self, struct_def: &'a StructDef) {
        if struct_def.fields().is_empty() {
            return;
        }
        let value_type = generate_struct_def_code_with_enums(struct_def, self.generated_types);
        quote_in! { self.props_tokens =>
            public $(&value_type) Value { get; set; }$['\r']
        };
        quote_in! { self.encode_tokens =>
            result.AddRange(this.Value.Encode());$['\r']
        };
        quote_in! { self.decode_tokens =>
            this.Value = new $(&value_type)();$['\r']
            this.Value.Decode(byteArray, ref p);$['\r']
        };
    }
}

impl<'a> Visitor<'a> for StructDefGenerator<'a> {
    fn visit_struct_def(&mut self, struct_def: &'a StructDef) {
        let is_regular_struct = struct_def.fields().iter().all(|f| f.name().is_some());
        let is_tuple_struct = struct_def.fields().iter().all(|f| f.name().is_none());
        if !is_regular_struct && !is_tuple_struct {
            panic!("Struct must be either regular or tuple");
        }
        self.is_tuple_struct = is_tuple_struct;

        if is_tuple_struct {
            self.tuple_struct(struct_def);
            return;
        }
        visitor::accept_struct_def(struct_def, self);
    }

    fn visit_struct_field(&mut self, struct_field: &'a StructField) {
        let type_decl_code =
            generate_type_decl_code_with_enums(struct_field.type_decl(), self.generated_types);

        self.props_tokens.push();
        self.props_tokens
            .append(summary_comment(struct_field.docs()));
        self.props_tokens.push();
        if let Some(field_name) = struct_field.name() {
            let field_name_pascal = field_name.to_case(Case::Pascal);
            quote_in! { self.props_tokens =>
                public $(&type_decl_code) $(&field_name_pascal) { get; set; }$['\r']
            };
            quote_in! { self.encode_tokens =>
                result.AddRange(this.$(&field_name_pascal).Encode());$['\r']
            };
            quote_in! { self.decode_tokens =>
                this.$(&field_name_pascal) = new $(&type_decl_code)();$['\r']
                this.$(&field_name_pascal).Decode(byteArray, ref p);$['\r']
            };
        }
    }
}

struct EnumDefGenerator<'a> {
    type_name: &'a str,
    generated_types: &'a Vec<&'a Type>,
    enum_tokens: Tokens,
    class_tokens: Tokens,
}

impl<'a> EnumDefGenerator<'a> {
    pub(crate) fn new(type_name: &'a str, generated_types: &'a Vec<&'a Type>) -> Self {
        Self {
            type_name,
            generated_types,
            enum_tokens: Tokens::new(),
            class_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let class_name = format!("Enum{}", self.type_name);
        quote!(
            public enum $(self.type_name)
            {
                $(self.enum_tokens)
            }

            public sealed partial class $(&class_name) : global::Substrate.NetApi.Model.Types.Base.BaseEnumRust<$(self.type_name)>
            {
                public $(&class_name)()
                {
                    $(self.class_tokens)
                }
            }
        )
    }
}

impl<'ast> Visitor<'ast> for EnumDefGenerator<'ast> {
    fn visit_enum_variant(&mut self, enum_variant: &'ast EnumVariant) {
        quote_in! { self.enum_tokens =>
            $(summary_comment(enum_variant.docs()))
        };
        quote_in! { self.enum_tokens =>
            $(enum_variant.name()),$['\r']
        };

        let type_decl_code = if let Some(type_decl) = enum_variant.type_decl().as_ref() {
            generate_type_decl_code_with_enums(type_decl, self.generated_types)
        } else {
            primitive_type_to_dotnet(PrimitiveType::Null).into()
        };
        quote_in! { self.class_tokens =>
            this.AddTypeDecoder<$(type_decl_code)>($(self.type_name).$(enum_variant.name()));$['\r']
        }
    }
}

pub(crate) struct TypeDeclGenerator<'a> {
    code: String,
    generated_types: &'a Vec<&'a Type>,
}

impl<'a> TypeDeclGenerator<'a> {
    pub(crate) fn new(generated_types: &'a Vec<&'a Type>) -> Self {
        Self {
            code: String::new(),
            generated_types,
        }
    }

    pub(crate) fn generate_type_decl(&mut self, type_decl: &'a TypeDecl) -> String {
        visitor::accept_type_decl(type_decl, self);
        let code = std::mem::take(&mut self.code);
        code
    }

    pub(crate) fn generate_types_as_tuple(&mut self, type_decls: Vec<&'a TypeDecl>) -> String {
        if type_decls.is_empty() {
        } else if type_decls.len() == 1 {
            visitor::accept_type_decl(type_decls[0], self);
        } else {
            self.code
                .push_str("global::Substrate.NetApi.Model.Types.Base.BaseTuple<");
            self.join_type_decls(type_decls, ", ");
            self.code.push('>');
        }
        let code = std::mem::take(&mut self.code);
        code
    }

    fn join_type_decls<I: IntoIterator<Item = &'a TypeDecl>>(
        &mut self,
        type_decls: I,
        separator: &str,
    ) {
        let prev_code = std::mem::take(&mut self.code);
        let mut type_decls_str: Vec<String> = Vec::new();
        let iter = type_decls.into_iter();
        for type_decl in iter {
            visitor::accept_type_decl(type_decl, self);
            type_decls_str.push(std::mem::take(&mut self.code));
        }
        _ = std::mem::replace(&mut self.code, prev_code);
        self.code.push_str(type_decls_str.join(separator).as_str());
    }
}

impl<'a> Visitor<'a> for TypeDeclGenerator<'a> {
    fn visit_optional_type_decl(&mut self, optional_type_decl: &'a TypeDecl) {
        self.code
            .push_str("global::Substrate.NetApi.Model.Types.Base.BaseOpt<");
        visitor::accept_type_decl(optional_type_decl, self);
        self.code.push('>');
    }

    fn visit_result_type_decl(&mut self, ok_type_decl: &'a TypeDecl, err_type_decl: &'a TypeDecl) {
        self.code.push_str("Result<");
        visitor::accept_type_decl(ok_type_decl, self);
        self.code.push_str(", ");
        visitor::accept_type_decl(err_type_decl, self);
        self.code.push('>');
    }

    fn visit_vector_type_decl(&mut self, vector_type_decl: &'a TypeDecl) {
        self.code
            .push_str("global::Substrate.NetApi.Model.Types.Base.BaseVec<");
        visitor::accept_type_decl(vector_type_decl, self);
        self.code.push('>');
    }

    fn visit_struct_def(&mut self, struct_def: &'a StructDef) {
        if struct_def.fields().is_empty() {
            return;
        }
        if struct_def.fields().len() == 1 {
            visitor::accept_type_decl(struct_def.fields()[0].type_decl(), self);
        } else {
            self.code
                .push_str("global::Substrate.NetApi.Model.Types.Base.BaseTuple<");
            self.join_type_decls(struct_def.fields().iter().map(|f| f.type_decl()), ", ");
            self.code.push('>');
        }
    }

    fn visit_primitive_type_id(&mut self, primitive_type_id: PrimitiveType) {
        self.code
            .push_str(primitive_type_to_dotnet(primitive_type_id));
    }

    fn visit_user_defined_type_id(&mut self, user_defined_type_id: &'a str) {
        let is_enum = self
            .generated_types
            .iter()
            .find(|&&t| t.name() == user_defined_type_id)
            .map(|&t| matches!(t.def(), TypeDef::Enum(_)))
            .unwrap_or_default();
        let type_id = if is_enum {
            &format!("Enum{}", user_defined_type_id)
        } else {
            user_defined_type_id
        };
        self.code.push_str(type_id);
    }

    fn visit_map_type_decl(&mut self, key_type_decl: &'a TypeDecl, value_type_decl: &'a TypeDecl) {
        self.code.push_str("BTreeMap<");
        visitor::accept_type_decl(key_type_decl, self);
        self.code.push_str(", ");
        visitor::accept_type_decl(value_type_decl, self);
        self.code.push('>');
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'a TypeDecl, len: u32) {
        self.code.push('[');
        visitor::accept_type_decl(item_type_decl, self);
        self.code.push_str(&format!("; {len}]"));
    }
}

fn primitive_type_to_dotnet(primitive_type: PrimitiveType) -> &'static str {
    match primitive_type {
        PrimitiveType::U8 => "global::Substrate.NetApi.Model.Types.Primitive.U8",
        PrimitiveType::U16 => "global::Substrate.NetApi.Model.Types.Primitive.U16",
        PrimitiveType::U32 => "global::Substrate.NetApi.Model.Types.Primitive.U32",
        PrimitiveType::U64 => "global::Substrate.NetApi.Model.Types.Primitive.U64",
        PrimitiveType::U128 => "global::Substrate.NetApi.Model.Types.Primitive.U128",
        PrimitiveType::I8 => "global::Substrate.NetApi.Model.Types.Primitive.I8",
        PrimitiveType::I16 => "global::Substrate.NetApi.Model.Types.Primitive.I16",
        PrimitiveType::I32 => "global::Substrate.NetApi.Model.Types.Primitive.I32",
        PrimitiveType::I64 => "global::Substrate.NetApi.Model.Types.Primitive.I64",
        PrimitiveType::I128 => "global::Substrate.NetApi.Model.Types.Primitive.I128",
        PrimitiveType::Bool => "global::Substrate.NetApi.Model.Types.Primitive.Bool",
        PrimitiveType::Str => "global::Substrate.NetApi.Model.Types.Primitive.Str",
        PrimitiveType::Char => "global::Substrate.NetApi.Model.Types.Primitive.PrimChar",
        PrimitiveType::Null => "global::Substrate.NetApi.Model.Types.Base.BaseVoid",
        PrimitiveType::ActorId => "global::Substrate.Gear.Api.Generated.Model.gprimitives.ActorId",
        PrimitiveType::CodeId => "global::Substrate.Gear.Api.Generated.Model.gprimitives.CodeId",
        PrimitiveType::MessageId => {
            "global::Substrate.Gear.Api.Generated.Model.gprimitives.MessageId"
        }
        PrimitiveType::H160 => "H160",
        PrimitiveType::H256 => "H256",
        PrimitiveType::U256 => "U256",
        PrimitiveType::NonZeroU8 => "NonZeroU8",
        PrimitiveType::NonZeroU16 => "NonZeroU16",
        PrimitiveType::NonZeroU32 => "NonZeroU32",
        PrimitiveType::NonZeroU64 => "NonZeroU64",
        PrimitiveType::NonZeroU128 => "NonZeroU128",
        PrimitiveType::NonZeroU256 => "NonZeroU256",
    }
}
