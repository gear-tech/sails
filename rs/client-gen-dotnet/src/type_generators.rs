use crate::helpers::summary_comment;
use convert_case::{Case, Casing};
use csharp::{block_comment, Tokens};
use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) fn generate_type_decl_code(type_decl: &TypeDecl) -> String {
    let mut type_decl_generator = TypeDeclGenerator::default();
    visitor::accept_type_decl(type_decl, &mut type_decl_generator);
    type_decl_generator.code
}

pub(crate) fn generate_type_decl_with_path(type_decl: &TypeDecl, path: String) -> String {
    let mut type_decl_generator = TypeDeclGenerator {
        code: String::new(),
        path,
    };
    visitor::accept_type_decl(type_decl, &mut type_decl_generator);
    type_decl_generator.code
}

pub(crate) struct TopLevelTypeGenerator<'a> {
    type_name: &'a str,
    tokens: Tokens,
}

impl<'a> TopLevelTypeGenerator<'a> {
    pub(crate) fn new(type_name: &'a str) -> Self {
        Self {
            type_name,
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
        let mut struct_def_generator = StructDefGenerator::new(self.type_name);
        struct_def_generator.visit_struct_def(struct_def);
        self.tokens.extend(struct_def_generator.finalize());
    }

    fn visit_enum_def(&mut self, enum_def: &'a EnumDef) {
        let mut enum_def_generator = EnumDefGenerator::new(self.type_name);
        enum_def_generator.visit_enum_def(enum_def);
        self.tokens.extend(enum_def_generator.finalize());
    }
}

#[derive(Default)]
struct StructDefGenerator<'a> {
    type_name: &'a str,
    is_tuple_struct: bool,
    props_tokens: Tokens,
    encode_tokens: Tokens,
    decode_tokens: Tokens,
}

impl<'a> StructDefGenerator<'a> {
    fn new(type_name: &'a str) -> Self {
        Self {
            type_name,
            is_tuple_struct: false,
            props_tokens: Tokens::new(),
            encode_tokens: Tokens::new(),
            decode_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        if self.is_tuple_struct {
            Tokens::new()
        } else {
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
                        global::System.Array.Copy(byteArray, start, Bytes, 0, bytesLength);
                    }
                }
            }
        }
    }
}

impl<'ast> Visitor<'ast> for StructDefGenerator<'ast> {
    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let is_regular_struct = struct_def.fields().iter().all(|f| f.name().is_some());
        let is_tuple_struct = struct_def.fields().iter().all(|f| f.name().is_none());
        if !is_regular_struct && !is_tuple_struct {
            panic!("Struct must be either regular or tuple");
        }
        self.is_tuple_struct = is_tuple_struct;

        visitor::accept_struct_def(struct_def, self);
    }

    fn visit_struct_field(&mut self, struct_field: &'ast StructField) {
        let type_decl_code = generate_type_decl_code(struct_field.type_decl());

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
                result.AddRange($(&field_name_pascal).Encode());$['\r']
            };
            quote_in! { self.decode_tokens =>
                $(&field_name_pascal) = new $(&type_decl_code)();$['\r']
                $(&field_name_pascal).Decode(byteArray, ref p);$['\r']
            };
        } else {
            quote_in! { self.props_tokens =>
                $(&type_decl_code),
            };
        }
    }
}

#[derive(Default)]
struct EnumDefGenerator<'a> {
    type_name: &'a str,
    enum_tokens: Tokens,
    class_tokens: Tokens,
}

impl<'a> EnumDefGenerator<'a> {
    pub(crate) fn new(type_name: &'a str) -> Self {
        Self {
            type_name,
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

            public sealed class $(&class_name) : global::Substrate.NetApi.Model.Types.Base.BaseEnumRust<$(self.type_name)>
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
            $(enum_variant.name()),
        };

        let type_decl_code = if let Some(type_decl) = enum_variant.type_decl().as_ref() {
            generate_type_decl_code(type_decl)
        } else {
            primitive_type_to_dotnet(PrimitiveType::Null).into()
        };
        quote_in! { self.class_tokens =>
            this.AddTypeDecoder<$(type_decl_code)>($(self.type_name).$(enum_variant.name()));$['\r']
        }
    }
}

#[derive(Default)]
struct TypeDeclGenerator {
    code: String,
    path: String,
}

impl<'ast> Visitor<'ast> for TypeDeclGenerator {
    fn visit_optional_type_decl(&mut self, optional_type_decl: &'ast TypeDecl) {
        self.code
            .push_str("global::Substrate.NetApi.Model.Types.Base.BaseOpt<");
        visitor::accept_type_decl(optional_type_decl, self);
        self.code.push('>');
    }

    fn visit_result_type_decl(
        &mut self,
        ok_type_decl: &'ast TypeDecl,
        err_type_decl: &'ast TypeDecl,
    ) {
        self.code.push_str("Result<");
        visitor::accept_type_decl(ok_type_decl, self);
        self.code.push_str(", ");
        visitor::accept_type_decl(err_type_decl, self);
        self.code.push('>');
    }

    fn visit_vector_type_decl(&mut self, vector_type_decl: &'ast TypeDecl) {
        self.code.push_str("Vec<");
        visitor::accept_type_decl(vector_type_decl, self);
        self.code.push('>');
    }

    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let mut struct_def_generator = StructTypeGenerator::new(self.path.clone());
        struct_def_generator.visit_struct_def(struct_def);
        self.code.push_str(&struct_def_generator.code);
    }

    fn visit_primitive_type_id(&mut self, primitive_type_id: PrimitiveType) {
        self.code
            .push_str(primitive_type_to_dotnet(primitive_type_id));
    }

    fn visit_user_defined_type_id(&mut self, user_defined_type_id: &'ast str) {
        if !self.path.is_empty() {
            self.code.push_str(&self.path);
            self.code.push_str("::");
        }
        self.code.push_str(user_defined_type_id);
    }

    fn visit_map_type_decl(
        &mut self,
        key_type_decl: &'ast TypeDecl,
        value_type_decl: &'ast TypeDecl,
    ) {
        self.code.push_str("BTreeMap<");
        visitor::accept_type_decl(key_type_decl, self);
        self.code.push_str(", ");
        visitor::accept_type_decl(value_type_decl, self);
        self.code.push('>');
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'ast TypeDecl, len: u32) {
        self.code.push('[');
        visitor::accept_type_decl(item_type_decl, self);
        self.code.push_str(&format!("; {len}]"));
    }
}

struct StructTypeGenerator {
    code: String,
    path: String,
}

impl StructTypeGenerator {
    fn new(path: String) -> Self {
        Self {
            code: String::new(),
            path,
        }
    }
}

impl<'ast> Visitor<'ast> for StructTypeGenerator {
    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let is_regular_struct = struct_def.fields().iter().all(|f| f.name().is_some());
        let is_tuple_struct = struct_def.fields().iter().all(|f| f.name().is_none());
        if !is_regular_struct && !is_tuple_struct {
            panic!("Struct must be either regular or tuple");
        }
        if is_regular_struct {
            self.code.push('{');
        } else {
            self.code.push('(');
        }
        visitor::accept_struct_def(struct_def, self);
        if is_regular_struct {
            self.code.push('}');
        } else {
            self.code.push(')');
        }
    }

    fn visit_struct_field(&mut self, struct_field: &'ast StructField) {
        let type_decl_code =
            generate_type_decl_with_path(struct_field.type_decl(), self.path.clone());

        if let Some(field_name) = struct_field.name() {
            self.code
                .push_str(&format!("{field_name}: {type_decl_code},"));
        } else {
            self.code.push_str(&format!("{type_decl_code},"));
        }
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
        PrimitiveType::ActorId => "ActorId",
        PrimitiveType::CodeId => "CodeId",
        PrimitiveType::MessageId => "MessageId",
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
