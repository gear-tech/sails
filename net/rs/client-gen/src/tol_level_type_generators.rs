use crate::{
    helpers::*,
    type_decl_generators::{primitive_type_to_dotnet, TypeDeclGenerator},
};
use convert_case::{Case, Casing};
use csharp::Tokens;
use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) struct TopLevelTypeGenerator<'a> {
    type_name: &'a str,
    type_generator: TypeDeclGenerator<'a>,
    tokens: Tokens,
}

impl<'a> TopLevelTypeGenerator<'a> {
    pub(crate) fn new(type_name: &'a str, type_generator: TypeDeclGenerator<'a>) -> Self {
        Self {
            type_name,
            type_generator,
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
            StructDefGenerator::new(self.type_name, self.type_generator.clone());
        struct_def_generator.visit_struct_def(struct_def);
        self.tokens.extend(struct_def_generator.finalize());
    }

    fn visit_enum_def(&mut self, enum_def: &'a EnumDef) {
        let mut enum_def_generator =
            EnumDefGenerator::new(self.type_name, self.type_generator.clone());
        enum_def_generator.visit_enum_def(enum_def);
        self.tokens.extend(enum_def_generator.finalize());
    }
}

struct StructDefGenerator<'a> {
    type_name: &'a str,
    type_generator: TypeDeclGenerator<'a>,
    is_tuple_struct: bool,
    props_tokens: Tokens,
    ensure_tokens: Tokens,
    encode_tokens: Tokens,
    decode_tokens: Tokens,
}

impl<'a> StructDefGenerator<'a> {
    fn new(type_name: &'a str, type_generator: TypeDeclGenerator<'a>) -> Self {
        Self {
            type_name,
            type_generator,
            is_tuple_struct: false,
            props_tokens: Tokens::new(),
            ensure_tokens: Tokens::new(),
            encode_tokens: Tokens::new(),
            decode_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let system_array = &csharp::import("global::System", "Array");
        let generic_list = &csharp::import("global::System.Collections.Generic", "List");

        quote! {
            public sealed partial class $(self.type_name) : global::Substrate.NetApi.Model.Types.Base.BaseType
            {
                $(self.props_tokens)

                $(inheritdoc())
                public override string TypeName() => $(quoted(self.type_name));

                $(inheritdoc())
                public override byte[] Encode()
                {
                    $(self.ensure_tokens)
                    var result = new $generic_list<byte>();$['\r']
                    $(self.encode_tokens)
                    return result.ToArray();$['\r']
                }

                $(inheritdoc())
                public override void Decode(byte[] byteArray, ref int p)
                {
                    var start = p;$['\r']
                    $(self.decode_tokens)
                    var bytesLength = p - start;
                    this.TypeSize = bytesLength;
                    this.Bytes = new byte[bytesLength];
                    $system_array.Copy(byteArray, start, this.Bytes, 0, bytesLength);
                }
            }
        }
    }

    fn tuple_struct(&mut self, struct_def: &'a StructDef) {
        if struct_def.fields().is_empty() {
            return;
        }
        let value_type = &self.type_generator.generate_struct_def(struct_def);
        quote_in! { self.props_tokens =>
            public $value_type? Value { get; set; }$['\r']
        };
        quote_in! { self.ensure_tokens =>
            if (this.Value is null)
            {
                throw new ArgumentNullException(nameof(this.Value), "Property cannot be null");
            }$['\r']
        };
        quote_in! { self.encode_tokens =>
            result.AddRange(this.Value!.Encode());$['\r']
        };
        quote_in! { self.decode_tokens =>
            this.Value = new $value_type();$['\r']
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
        } else {
            visitor::accept_struct_def(struct_def, self);
        }
    }

    fn visit_struct_field(&mut self, struct_field: &'a StructField) {
        let type_decl_code = &self
            .type_generator
            .generate_type_decl(struct_field.type_decl());

        self.props_tokens.push();
        self.props_tokens
            .append(summary_comment(struct_field.docs()));
        self.props_tokens.push();
        if let Some(field_name) = struct_field.name() {
            let field_name_pascal = &field_name.to_case(Case::Pascal);
            quote_in! { self.props_tokens =>
                public $type_decl_code? $field_name_pascal { get; set; }$['\r']
            };
            quote_in! { self.ensure_tokens =>
                if (this.$field_name_pascal is null)
                {
                    throw new ArgumentNullException(nameof(this.$field_name_pascal), "Property cannot be null");
                }$['\r']
            };
            quote_in! { self.encode_tokens =>
                result.AddRange(this.$field_name_pascal!.Encode());$['\r']
            };
            quote_in! { self.decode_tokens =>
                this.$field_name_pascal = new $type_decl_code();$['\r']
                this.$field_name_pascal.Decode(byteArray, ref p);$['\r']
            };
        }
    }
}

struct EnumDefGenerator<'a> {
    type_name: &'a str,
    type_generator: TypeDeclGenerator<'a>,
    enum_tokens: Tokens,
    class_tokens: Tokens,
}

impl<'a> EnumDefGenerator<'a> {
    pub(crate) fn new(type_name: &'a str, type_generator: TypeDeclGenerator<'a>) -> Self {
        Self {
            type_name,
            type_generator,
            enum_tokens: Tokens::new(),
            class_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let class_name = &format!("Enum{}", self.type_name);
        quote!(
            public enum $(self.type_name)
            {
                $(self.enum_tokens)
            }

            public sealed partial class $class_name : global::Substrate.NetApi.Model.Types.Base.BaseEnumRust<$(self.type_name)>
            {
                public $class_name()
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
            $(enum_variant.name()),$['\r']
        };

        let type_decl_code = if let Some(type_decl) = enum_variant.type_decl().as_ref() {
            self.type_generator.generate_type_decl(type_decl)
        } else {
            primitive_type_to_dotnet(PrimitiveType::Null).into()
        };
        quote_in! { self.class_tokens =>
            this.AddTypeDecoder<$(type_decl_code)>($(self.type_name).$(enum_variant.name()));$['\r']
        }
    }
}
