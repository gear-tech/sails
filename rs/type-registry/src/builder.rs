use crate::registry::TypeRef;
use crate::ty::{
    Annotation, Composite, Field, GenericArg, Primitive, Type, TypeDef, TypeParameter, Variant,
    VariantDef,
};
use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone)]
pub struct FieldBuilder<P> {
    parent: P,
    name: Option<String>,
    metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct VariantBuilder {
    parent: VariantDefBuilder,
    name: String,
    metadata: Metadata,
    fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct CompositeBuilder {
    type_builder: TypeBuilder,
    fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct VariantDefBuilder {
    type_builder: TypeBuilder,
    variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
pub struct ParamBuilder {
    inner: TypeBuilder,
    param_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct TypeBuilder {
    module_path: String,
    name: String,
    type_params: Vec<TypeParameter>,
    metadata: Metadata,
}

#[derive(Debug, Clone, Default)]
struct Metadata {
    docs: Vec<String>,
    annotations: Vec<Annotation>,
}

pub trait PushField: Sized {
    fn push_field(&mut self, field: Field);
}

impl<P: PushField> FieldBuilder<P> {
    pub fn ty(mut self, ty: TypeRef) -> P {
        self.parent.push_field(Field {
            name: self.name,
            ty,
            docs: self.metadata.docs,
            annotations: self.metadata.annotations,
        });
        self.parent
    }
}

impl<P> FieldBuilder<P> {
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.metadata.doc(doc);
        self
    }

    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        self.metadata.annotate(name);
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.metadata.value(value);
        self
    }
}

impl VariantBuilder {
    pub fn field(self, name: impl Into<String>) -> FieldBuilder<Self> {
        FieldBuilder {
            parent: self,
            name: Some(name.into()),
            metadata: Metadata::default(),
        }
    }

    pub fn unnamed(self) -> FieldBuilder<Self> {
        FieldBuilder {
            parent: self,
            name: None,
            metadata: Metadata::default(),
        }
    }

    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.metadata.doc(doc);
        self
    }

    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        self.metadata.annotate(name);
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.metadata.value(value);
        self
    }

    pub fn finish_variant(mut self) -> VariantDefBuilder {
        self.parent.variants.push(Variant {
            name: self.name,
            fields: self.fields,
            docs: self.metadata.docs,
            annotations: self.metadata.annotations,
        });
        self.parent
    }
}

impl CompositeBuilder {
    pub fn field(self, name: impl Into<String>) -> FieldBuilder<Self> {
        FieldBuilder {
            parent: self,
            name: Some(name.into()),
            metadata: Metadata::default(),
        }
    }

    pub fn unnamed(self) -> FieldBuilder<Self> {
        FieldBuilder {
            parent: self,
            name: None,
            metadata: Metadata::default(),
        }
    }

    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.type_builder.metadata.doc(doc);
        self
    }

    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        self.type_builder.metadata.annotate(name);
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.type_builder.metadata.value(value);
        self
    }

    pub fn build(self) -> Type {
        self.type_builder.build(TypeDef::Composite(Composite {
            fields: self.fields,
        }))
    }
}

impl VariantDefBuilder {
    pub fn add_variant(self, name: impl Into<String>) -> VariantBuilder {
        VariantBuilder {
            parent: self,
            name: name.into(),
            metadata: Metadata::default(),
            fields: Vec::new(),
        }
    }

    pub fn build(self) -> Type {
        self.type_builder.build(TypeDef::Variant(VariantDef {
            variants: self.variants,
        }))
    }
}

impl ParamBuilder {
    pub fn arg(mut self, arg: TypeRef) -> TypeBuilder {
        self.inner.type_params.push(TypeParameter {
            name: self.param_name,
            arg: GenericArg::Type(arg),
        });
        self.inner
    }

    pub fn val(mut self, val: impl Into<String>) -> TypeBuilder {
        self.inner.type_params.push(TypeParameter {
            name: self.param_name,
            arg: GenericArg::Const(val.into()),
        });
        self.inner
    }
}

impl TypeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn module_path(mut self, module_path: impl Into<String>) -> Self {
        self.module_path = module_path.into();
        self
    }

    pub fn param(self, name: impl Into<String>) -> ParamBuilder {
        ParamBuilder {
            inner: self,
            param_name: name.into(),
        }
    }

    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.metadata.doc(doc);
        self
    }

    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        self.metadata.annotate(name);
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.metadata.value(value);
        self
    }

    pub fn primitive(self, primitive: Primitive) -> Type {
        self.build(TypeDef::Primitive(primitive))
    }

    #[cfg(feature = "gprimitives")]
    pub fn gprimitive(self, gprimitive: crate::ty::GPrimitive) -> Type {
        self.build(TypeDef::GPrimitive(gprimitive))
    }

    pub fn array(self, type_param: TypeRef, len: u32) -> Type {
        self.build(TypeDef::Array { len, type_param })
    }

    pub fn sequence(self, type_param: TypeRef) -> Type {
        self.build(TypeDef::Sequence(type_param))
    }

    pub fn tuple(self, elems: Vec<TypeRef>) -> Type {
        self.build(TypeDef::Tuple(elems))
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

    pub fn parameter(self, name: impl Into<String>) -> Type {
        self.build(TypeDef::Parameter(name.into()))
    }

    pub fn applied(self, base: TypeRef, args: Vec<TypeRef>) -> Type {
        self.build(TypeDef::Applied { base, args })
    }

    pub fn composite(self) -> CompositeBuilder {
        CompositeBuilder {
            type_builder: self,
            fields: Vec::new(),
        }
    }

    pub fn variant(self) -> VariantDefBuilder {
        VariantDefBuilder {
            type_builder: self,
            variants: Vec::new(),
        }
    }

    fn build(self, def: TypeDef) -> Type {
        Type {
            module_path: self.module_path,
            name: self.name,
            type_params: self.type_params,
            def,
            docs: self.metadata.docs,
            annotations: self.metadata.annotations,
        }
    }
}

impl Metadata {
    fn doc(&mut self, doc: impl Into<String>) {
        self.docs.push(doc.into());
    }

    fn annotate(&mut self, name: impl Into<String>) {
        self.annotations.push(Annotation {
            name: name.into(),
            value: None,
        });
    }

    fn value(&mut self, value: impl Into<String>) {
        if let Some(ann) = self.annotations.last_mut() {
            ann.value = Some(value.into());
        }
    }
}

impl PushField for CompositeBuilder {
    fn push_field(&mut self, field: Field) {
        self.fields.push(field);
    }
}

impl PushField for VariantBuilder {
    fn push_field(&mut self, field: Field) {
        self.fields.push(field);
    }
}
