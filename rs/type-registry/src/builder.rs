use crate::registry::TypeRef;
use crate::ty::{
    Annotation, Composite, Field, FieldType, GenericArg, Primitive, Type, TypeDef, TypeDefinition,
    TypeDefinitionKind, TypeParameter, Variant, VariantDef,
};
use alloc::{string::String, vec::Vec};

pub struct TypeBuilder {
    module_path: String,
    name: String,
    type_params: Vec<TypeParameter>,
    docs: Vec<String>,
    annotations: Vec<Annotation>,
}

impl Default for TypeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeBuilder {
    pub fn new() -> Self {
        Self {
            module_path: String::new(),
            name: String::new(),
            type_params: Vec::new(),
            docs: Vec::new(),
            annotations: Vec::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn module_path(mut self, module_path: impl Into<String>) -> Self {
        self.module_path = module_path.into();
        self
    }

    pub fn type_param(mut self, name: impl Into<String>, arg: TypeRef) -> Self {
        self.type_params.push(TypeParameter {
            name: name.into(),
            arg: GenericArg::Type(arg),
        });
        self
    }

    pub fn const_param(mut self, name: impl Into<String>, arg: impl Into<String>) -> Self {
        self.type_params.push(TypeParameter {
            name: name.into(),
            arg: GenericArg::Const(arg.into()),
        });
        self
    }

    pub fn docs(mut self, docs: Vec<String>) -> Self {
        self.docs = docs;
        self
    }

    pub fn annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    fn build(self, def: TypeDef) -> Type {
        Type {
            module_path: self.module_path,
            name: self.name,
            type_params: self.type_params,
            def,
            docs: self.docs,
            annotations: self.annotations,
        }
    }

    pub fn composite(self) -> CompositeBuilder {
        CompositeBuilder::new(self)
    }

    pub fn variant(self) -> VariantDefBuilder {
        VariantDefBuilder::new(self)
    }

    pub(crate) fn build_composite(self, composite: Composite) -> Type {
        let def = TypeDef::Definition(TypeDefinition {
            kind: TypeDefinitionKind::Composite(composite),
        });
        self.build(def)
    }

    pub(crate) fn build_variant(self, variant: VariantDef) -> Type {
        let def = TypeDef::Definition(TypeDefinition {
            kind: TypeDefinitionKind::Variant(variant),
        });
        self.build(def)
    }

    pub fn primitive(self, primitive: Primitive) -> Type {
        self.build(TypeDef::Primitive(primitive))
    }

    #[cfg(feature = "gprimitives")]
    pub fn gprimitive(self, gprimitive: crate::ty::GPrimitive) -> Type {
        self.build(TypeDef::GPrimitive(gprimitive))
    }

    pub fn sequence(self, type_param: TypeRef) -> Type {
        self.build(TypeDef::Sequence(type_param))
    }

    pub fn array(self, len: u32, type_param: TypeRef) -> Type {
        self.build(TypeDef::Array { len, type_param })
    }

    pub fn tuple(self, fields: Vec<TypeRef>) -> Type {
        self.build(TypeDef::Tuple(fields))
    }

    pub fn map(self, key: TypeRef, value: TypeRef) -> Type {
        self.build(TypeDef::Map { key, value })
    }

    pub fn option(self, type_param: TypeRef) -> Type {
        self.build(TypeDef::Option(type_param))
    }

    pub fn result(self, ok: TypeRef, err: TypeRef) -> Type {
        self.build(TypeDef::Result { ok, err })
    }
}

pub struct CompositeBuilder {
    type_builder: TypeBuilder,
    fields: Vec<Field>,
}

impl CompositeBuilder {
    pub fn new(type_builder: TypeBuilder) -> Self {
        Self {
            type_builder,
            fields: Vec::new(),
        }
    }

    pub fn field(mut self, name: impl Into<String>, ty: impl Into<FieldType>) -> Self {
        self.fields.push(Field {
            name: Some(name.into()),
            ty: ty.into(),
            type_name: None,
            docs: Vec::new(),
            annotations: Vec::new(),
        });
        self
    }

    pub fn unnamed_field(mut self, ty: impl Into<FieldType>) -> Self {
        self.fields.push(Field {
            name: None,
            ty: ty.into(),
            type_name: None,
            docs: Vec::new(),
            annotations: Vec::new(),
        });
        self
    }

    pub fn type_name(mut self, type_name: impl Into<String>) -> Self {
        if let Some(field) = self.fields.last_mut() {
            field.type_name = Some(type_name.into());
        }
        self
    }

    pub fn docs(mut self, docs: Vec<String>) -> Self {
        if let Some(field) = self.fields.last_mut() {
            field.docs = docs;
        }
        self
    }

    pub fn annotation(mut self, annotation: Annotation) -> Self {
        if let Some(field) = self.fields.last_mut() {
            field.annotations.push(annotation);
        }
        self
    }

    pub fn fields(mut self, mut fields: Vec<Field>) -> Self {
        self.fields.append(&mut fields);
        self
    }

    pub fn build(self) -> Type {
        self.type_builder.build_composite(Composite {
            fields: self.fields,
        })
    }
}

pub struct VariantDefBuilder {
    type_builder: TypeBuilder,
    variants: Vec<Variant>,
}

impl VariantDefBuilder {
    pub fn new(type_builder: TypeBuilder) -> Self {
        Self {
            type_builder,
            variants: Vec::new(),
        }
    }

    pub fn add_variant(self, name: impl Into<String>) -> VariantBuilder {
        VariantBuilder::new(self, name.into())
    }

    pub fn build(self) -> Type {
        self.type_builder.build_variant(VariantDef {
            variants: self.variants,
        })
    }
}

pub struct VariantBuilder {
    parent: VariantDefBuilder,
    variant: Variant,
}

impl VariantBuilder {
    fn new(parent: VariantDefBuilder, name: String) -> Self {
        Self {
            parent,
            variant: Variant {
                name,
                fields: Vec::new(),
                docs: Vec::new(),
                annotations: Vec::new(),
            },
        }
    }

    pub fn docs(mut self, docs: Vec<String>) -> Self {
        self.variant.docs = docs;
        self
    }

    pub fn annotation(mut self, annotation: Annotation) -> Self {
        self.variant.annotations.push(annotation);
        self
    }

    pub fn field(mut self, name: impl Into<String>, ty: impl Into<FieldType>) -> Self {
        self.variant.fields.push(Field {
            name: Some(name.into()),
            ty: ty.into(),
            type_name: None,
            docs: Vec::new(),
            annotations: Vec::new(),
        });
        self
    }

    pub fn unnamed_field(mut self, ty: impl Into<FieldType>) -> Self {
        self.variant.fields.push(Field {
            name: None,
            ty: ty.into(),
            type_name: None,
            docs: Vec::new(),
            annotations: Vec::new(),
        });
        self
    }

    pub fn type_name(mut self, type_name: impl Into<String>) -> Self {
        if let Some(field) = self.variant.fields.last_mut() {
            field.type_name = Some(type_name.into());
        }
        self
    }

    pub fn field_docs(mut self, docs: Vec<String>) -> Self {
        if let Some(field) = self.variant.fields.last_mut() {
            field.docs = docs;
        }
        self
    }

    pub fn field_annotation(mut self, annotation: Annotation) -> Self {
        if let Some(field) = self.variant.fields.last_mut() {
            field.annotations.push(annotation);
        }
        self
    }

    pub fn add_variant(mut self, name: impl Into<String>) -> VariantBuilder {
        self.parent.variants.push(self.variant);
        VariantBuilder::new(self.parent, name.into())
    }

    pub fn build(mut self) -> Type {
        self.parent.variants.push(self.variant);
        self.parent.build()
    }
}
