use crate::registry::TypeRef;
use crate::ty::{
    Annotation, Composite, Field, GenericArg, Primitive, Type, TypeDef, TypeParameter, Variant,
    VariantDef,
};
use alloc::{string::String, vec::Vec};

#[derive(Default)]
pub struct TypeBuilder {
    module_path: String,
    name: String,
    type_params: Vec<TypeParameter>,
    docs: Vec<String>,
    annotations: Vec<Annotation>,
    pending_param_name: Option<String>,
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
        self.docs.push(doc.into());
        self
    }

    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        self.annotations.push(Annotation {
            name: name.into(),
            value: None,
        });
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        if let Some(ann) = self.annotations.last_mut() {
            ann.value = Some(value.into());
        }
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
            current_name: None,
            current_docs: Vec::new(),
            current_annotations: Vec::new(),
            current_type_name: None,
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
            docs: self.docs,
            annotations: self.annotations,
        }
    }
}

pub struct CompositeBuilder {
    type_builder: TypeBuilder,
    fields: Vec<Field>,
    /// None: idle; Some(Some(name)): building named field; Some(None): building unnamed field
    current_name: Option<Option<String>>,
    current_docs: Vec<String>,
    current_annotations: Vec<Annotation>,
    current_type_name: Option<String>,
}

impl CompositeBuilder {
    pub fn field(mut self, name: impl Into<String>) -> Self {
        self.current_name = Some(Some(name.into()));
        self
    }

    pub fn unnamed(mut self) -> Self {
        self.current_name = Some(None);
        self
    }

    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        if self.current_name.is_some() {
            self.current_docs.push(doc.into());
        } else {
            self.type_builder = self.type_builder.doc(doc);
        }
        self
    }

    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        if self.current_name.is_some() {
            self.current_annotations.push(Annotation {
                name: name.into(),
                value: None,
            });
        } else {
            self.type_builder = self.type_builder.annotate(name);
        }
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        if self.current_name.is_some() {
            if let Some(ann) = self.current_annotations.last_mut() {
                ann.value = Some(value.into());
            }
        } else {
            self.type_builder = self.type_builder.value(value);
        }
        self
    }

    pub fn type_name(mut self, type_name: impl Into<String>) -> Self {
        self.current_type_name = Some(type_name.into());
        self
    }

    pub fn ty(mut self, ty: TypeRef) -> Self {
        if let Some(name) = self.current_name.take() {
            self.fields.push(Field {
                name,
                ty,
                type_name: self.current_type_name.take(),
                docs: core::mem::take(&mut self.current_docs),
                annotations: core::mem::take(&mut self.current_annotations),
            });
        }
        self
    }

    pub fn build(self) -> Type {
        self.type_builder.build(TypeDef::Composite(Composite {
            fields: self.fields,
        }))
    }
}

pub struct VariantDefBuilder {
    pub(crate) type_builder: TypeBuilder,
    pub(crate) variants: Vec<Variant>,
}

impl VariantDefBuilder {
    pub fn add_variant(self, name: impl Into<String>) -> VariantBuilder {
        VariantBuilder {
            parent: self,
            variant_name: name.into(),
            fields: Vec::new(),
            variant_docs: Vec::new(),
            variant_annotations: Vec::new(),
            current_name: None,
            current_docs: Vec::new(),
            current_annotations: Vec::new(),
            current_type_name: None,
        }
    }

    pub fn build(self) -> Type {
        self.type_builder.build(TypeDef::Variant(VariantDef {
            variants: self.variants,
        }))
    }
}

pub struct VariantBuilder {
    parent: VariantDefBuilder,
    variant_name: String,
    fields: Vec<Field>,
    variant_docs: Vec<String>,
    variant_annotations: Vec<Annotation>,
    /// None: idle; Some(Some(name)): building named field; Some(None): building unnamed field
    current_name: Option<Option<String>>,
    current_docs: Vec<String>,
    current_annotations: Vec<Annotation>,
    current_type_name: Option<String>,
}

impl VariantBuilder {
    pub fn field(mut self, name: impl Into<String>) -> Self {
        self.current_name = Some(Some(name.into()));
        self
    }

    pub fn unnamed(mut self) -> Self {
        self.current_name = Some(None);
        self
    }

    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        if self.current_name.is_some() {
            self.current_docs.push(doc.into());
        } else {
            self.variant_docs.push(doc.into());
        }
        self
    }

    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        if self.current_name.is_some() {
            self.current_annotations.push(Annotation {
                name: name.into(),
                value: None,
            });
        } else {
            self.variant_annotations.push(Annotation {
                name: name.into(),
                value: None,
            });
        }
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        if self.current_name.is_some() {
            if let Some(ann) = self.current_annotations.last_mut() {
                ann.value = Some(value.into());
            }
        } else if let Some(ann) = self.variant_annotations.last_mut() {
            ann.value = Some(value.into());
        }
        self
    }

    pub fn type_name(mut self, type_name: impl Into<String>) -> Self {
        self.current_type_name = Some(type_name.into());
        self
    }

    pub fn ty(mut self, ty: TypeRef) -> Self {
        if let Some(name) = self.current_name.take() {
            self.fields.push(Field {
                name,
                ty,
                type_name: self.current_type_name.take(),
                docs: core::mem::take(&mut self.current_docs),
                annotations: core::mem::take(&mut self.current_annotations),
            });
        }
        self
    }

    pub fn finish_variant(mut self) -> VariantDefBuilder {
        self.parent.variants.push(Variant {
            name: self.variant_name,
            fields: self.fields,
            docs: self.variant_docs,
            annotations: self.variant_annotations,
        });
        self.parent
    }

    pub fn build(self) -> Type {
        let parent = self.finish_variant();
        parent.build()
    }
}
