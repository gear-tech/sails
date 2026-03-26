use crate::registry::TypeRef;
use crate::ty::{
    Annotation, Composite, Field, GenericArg, Primitive, Type, TypeDef, TypeParameter, Variant,
    VariantDef,
};
use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone, Default)]
pub struct TypeBuilder {
    module_path: String,
    name: String,
    type_params: Vec<TypeParameter>,
    metadata: Metadata,
    pending_param_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VariantBuilder {
    fields_builder: FieldsBuilder<VariantDefBuilder>,
    name: String,
    metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct CompositeBuilder {
    fields_builder: FieldsBuilder<TypeBuilder>,
}

#[derive(Debug, Clone)]
pub struct VariantDefBuilder {
    type_builder: TypeBuilder,
    variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
struct FieldsBuilder<P> {
    parent: P,
    fields: Vec<Field>,
    current_name: Option<Option<String>>,
    current_metadata: Metadata,
    current_type_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct Metadata {
    docs: Vec<String>,
    annotations: Vec<Annotation>,
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

    pub fn type_param(mut self, name: impl Into<String>) -> Self {
        self.pending_param_name = Some(name.into());
        self
    }

    pub fn const_param(mut self, name: impl Into<String>) -> Self {
        self.pending_param_name = Some(name.into());
        self
    }

    pub fn arg(mut self, arg: TypeRef) -> Self {
        if let Some(name) = self.pending_param_name.take() {
            self.type_params.push(TypeParameter {
                name,
                arg: GenericArg::Type(arg),
            });
        }
        self
    }

    pub fn val(mut self, val: impl Into<String>) -> Self {
        if let Some(name) = self.pending_param_name.take() {
            self.type_params.push(TypeParameter {
                name,
                arg: GenericArg::Const(val.into()),
            });
        }
        self
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
            fields_builder: FieldsBuilder::new(self),
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

impl VariantBuilder {
    pub fn field(mut self, name: impl Into<String>) -> Self {
        self.fields_builder.field(name);
        self
    }

    pub fn unnamed(mut self) -> Self {
        self.fields_builder.unnamed();
        self
    }

    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        let doc = doc.into();
        if !self.fields_builder.try_doc(doc.clone()) {
            self.metadata.doc(doc);
        }
        self
    }

    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        if !self.fields_builder.try_annotate(name.clone()) {
            self.metadata.annotate(name);
        }
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if !self.fields_builder.try_value(value.clone()) {
            self.metadata.value(value);
        }
        self
    }

    pub fn type_name(mut self, type_name: impl Into<String>) -> Self {
        self.fields_builder.type_name(type_name);
        self
    }

    pub fn ty(mut self, ty: TypeRef) -> Self {
        self.fields_builder.ty(ty);
        self
    }

    pub fn finish_variant(self) -> VariantDefBuilder {
        let fields = self.fields_builder.fields;
        let mut parent = self.fields_builder.parent;
        parent.variants.push(Variant {
            name: self.name,
            fields,
            docs: self.metadata.docs,
            annotations: self.metadata.annotations,
        });
        parent
    }

    pub fn build(self) -> Type {
        self.finish_variant().build()
    }
}

impl CompositeBuilder {
    pub fn field(mut self, name: impl Into<String>) -> Self {
        self.fields_builder.field(name);
        self
    }

    pub fn unnamed(mut self) -> Self {
        self.fields_builder.unnamed();
        self
    }

    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        let doc = doc.into();
        if !self.fields_builder.try_doc(doc.clone()) {
            self.fields_builder.parent.metadata.doc(doc);
        }
        self
    }

    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        if !self.fields_builder.try_annotate(name.clone()) {
            self.fields_builder.parent.metadata.annotate(name);
        }
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if !self.fields_builder.try_value(value.clone()) {
            self.fields_builder.parent.metadata.value(value);
        }
        self
    }

    pub fn type_name(mut self, type_name: impl Into<String>) -> Self {
        self.fields_builder.type_name(type_name);
        self
    }

    pub fn ty(mut self, ty: TypeRef) -> Self {
        self.fields_builder.ty(ty);
        self
    }

    pub fn build(self) -> Type {
        let fields = self.fields_builder.fields;
        self.fields_builder
            .parent
            .build(TypeDef::Composite(Composite { fields }))
    }
}

impl VariantDefBuilder {
    pub fn add_variant(self, name: impl Into<String>) -> VariantBuilder {
        VariantBuilder {
            fields_builder: FieldsBuilder::new(self),
            name: name.into(),
            metadata: Metadata::default(),
        }
    }

    pub fn build(self) -> Type {
        self.type_builder.build(TypeDef::Variant(VariantDef {
            variants: self.variants,
        }))
    }
}

impl<P> FieldsBuilder<P> {
    fn new(parent: P) -> Self {
        Self {
            parent,
            fields: Vec::new(),
            current_name: None,
            current_metadata: Metadata::default(),
            current_type_name: None,
        }
    }

    fn field(&mut self, name: impl Into<String>) {
        self.current_name = Some(Some(name.into()));
    }

    fn unnamed(&mut self) {
        self.current_name = Some(None);
    }

    fn type_name(&mut self, type_name: impl Into<String>) {
        self.current_type_name = Some(type_name.into());
    }

    fn ty(&mut self, ty: TypeRef) {
        if let Some(name) = self.current_name.take() {
            self.fields.push(Field {
                name,
                ty,
                type_name: self.current_type_name.take(),
                docs: core::mem::take(&mut self.current_metadata.docs),
                annotations: core::mem::take(&mut self.current_metadata.annotations),
            });
        }
    }

    fn try_doc(&mut self, doc: String) -> bool {
        if self.current_name.is_some() {
            self.current_metadata.doc(doc);
            true
        } else {
            false
        }
    }

    fn try_annotate(&mut self, name: String) -> bool {
        if self.current_name.is_some() {
            self.current_metadata.annotate(name);
            true
        } else {
            false
        }
    }

    fn try_value(&mut self, value: String) -> bool {
        if self.current_name.is_some() {
            self.current_metadata.value(value);
            true
        } else {
            false
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
