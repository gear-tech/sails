use crate::registry::TypeRef;
use crate::ty::{
    Annotation, Composite, Field, GenericArg, Primitive, Type, TypeDef, TypeParameter, Variant,
    VariantDef,
};
use alloc::{string::String, vec::Vec};

/// Builder for a field attached to a composite or variant parent.
#[derive(Debug, Clone)]
pub struct FieldBuilder<P> {
    parent: P,
    name: Option<String>,
    metadata: Metadata,
}

/// Builder for a single enum variant.
#[derive(Debug, Clone)]
pub struct VariantBuilder {
    parent: VariantDefBuilder,
    name: String,
    metadata: Metadata,
    fields: Vec<Field>,
}

/// Builder for composite type definitions.
#[derive(Debug, Clone)]
pub struct CompositeBuilder {
    type_builder: TypeBuilder,
    fields: Vec<Field>,
}

/// Builder for variant type definitions.
#[derive(Debug, Clone)]
pub struct VariantDefBuilder {
    type_builder: TypeBuilder,
    variants: Vec<Variant>,
}

/// Builder for a single declared generic parameter.
#[derive(Debug, Clone)]
pub struct ParamBuilder {
    inner: TypeBuilder,
    param_name: String,
}

/// Entry-point builder for synthetic [`Type`] values.
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

/// Helper trait implemented by builders that can receive fields.
pub trait PushField: Sized {
    /// Appends a fully built field to the parent.
    fn push_field(&mut self, field: Field);
}

impl<P: PushField> FieldBuilder<P> {
    /// Completes the field by assigning its type and returning the parent.
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
    /// Appends a documentation line to the field.
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.metadata.doc(doc);
        self
    }

    /// Starts a new annotation on the field.
    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        self.metadata.annotate(name);
        self
    }

    /// Sets the value for the most recently added annotation.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.metadata.value(value);
        self
    }
}

impl VariantBuilder {
    /// Starts a named field inside the variant.
    pub fn field(self, name: impl Into<String>) -> FieldBuilder<Self> {
        FieldBuilder {
            parent: self,
            name: Some(name.into()),
            metadata: Metadata::default(),
        }
    }

    /// Starts an unnamed field inside the variant.
    pub fn unnamed(self) -> FieldBuilder<Self> {
        FieldBuilder {
            parent: self,
            name: None,
            metadata: Metadata::default(),
        }
    }

    /// Appends a documentation line to the variant.
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.metadata.doc(doc);
        self
    }

    /// Starts a new annotation on the variant.
    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        self.metadata.annotate(name);
        self
    }

    /// Sets the value for the most recently added annotation.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.metadata.value(value);
        self
    }

    /// Finishes the variant and returns to the enclosing variant builder.
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
    /// Starts a named field inside the composite.
    pub fn field(self, name: impl Into<String>) -> FieldBuilder<Self> {
        FieldBuilder {
            parent: self,
            name: Some(name.into()),
            metadata: Metadata::default(),
        }
    }

    /// Starts an unnamed field inside the composite.
    pub fn unnamed(self) -> FieldBuilder<Self> {
        FieldBuilder {
            parent: self,
            name: None,
            metadata: Metadata::default(),
        }
    }

    /// Appends a documentation line to the type being built.
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.type_builder.metadata.doc(doc);
        self
    }

    /// Starts a new annotation on the type being built.
    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        self.type_builder.metadata.annotate(name);
        self
    }

    /// Sets the value for the most recently added annotation.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.type_builder.metadata.value(value);
        self
    }

    /// Builds the final composite [`Type`].
    pub fn build(self) -> Type {
        self.type_builder.build(TypeDef::Composite(Composite {
            fields: self.fields,
        }))
    }
}

impl VariantDefBuilder {
    /// Starts a new variant inside the enum-like definition.
    pub fn add_variant(self, name: impl Into<String>) -> VariantBuilder {
        VariantBuilder {
            parent: self,
            name: name.into(),
            metadata: Metadata::default(),
            fields: Vec::new(),
        }
    }

    /// Builds the final variant [`Type`].
    pub fn build(self) -> Type {
        self.type_builder.build(TypeDef::Variant(VariantDef {
            variants: self.variants,
        }))
    }
}

impl ParamBuilder {
    /// Assigns a type argument to the declared generic parameter.
    pub fn arg(mut self, arg: TypeRef) -> TypeBuilder {
        self.inner.type_params.push(TypeParameter {
            name: self.param_name,
            arg: GenericArg::Type(arg),
        });
        self.inner
    }

    /// Assigns a const argument to the declared generic parameter.
    pub fn val(mut self, val: impl Into<String>) -> TypeBuilder {
        self.inner.type_params.push(TypeParameter {
            name: self.param_name,
            arg: GenericArg::Const(val.into()),
        });
        self.inner
    }
}

impl TypeBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the recorded type name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the recorded module path.
    pub fn module_path(mut self, module_path: impl Into<String>) -> Self {
        self.module_path = module_path.into();
        self
    }

    /// Declares a generic parameter on the type.
    pub fn param(self, name: impl Into<String>) -> ParamBuilder {
        ParamBuilder {
            inner: self,
            param_name: name.into(),
        }
    }

    /// Appends a documentation line to the type.
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.metadata.doc(doc);
        self
    }

    /// Starts a new annotation on the type.
    pub fn annotate(mut self, name: impl Into<String>) -> Self {
        self.metadata.annotate(name);
        self
    }

    /// Sets the value for the most recently added annotation.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.metadata.value(value);
        self
    }

    /// Builds a primitive type description.
    pub fn primitive(self, primitive: Primitive) -> Type {
        self.build(TypeDef::Primitive(primitive))
    }

    /// Builds a Gear primitive type description.
    #[cfg(feature = "gprimitives")]
    pub fn gprimitive(self, gprimitive: crate::ty::GPrimitive) -> Type {
        self.build(TypeDef::GPrimitive(gprimitive))
    }

    /// Builds a fixed-size array type description.
    pub fn array(self, type_param: TypeRef, len: u32) -> Type {
        self.build(TypeDef::Array { len, type_param })
    }

    /// Builds a sequence type description.
    pub fn sequence(self, type_param: TypeRef) -> Type {
        self.build(TypeDef::Sequence(type_param))
    }

    /// Builds a tuple type description.
    pub fn tuple(self, elems: Vec<TypeRef>) -> Type {
        self.build(TypeDef::Tuple(elems))
    }

    /// Builds a map type description.
    pub fn map(self, key: TypeRef, value: TypeRef) -> Type {
        self.build(TypeDef::Map { key, value })
    }

    /// Builds an option type description.
    pub fn option(self, type_param: TypeRef) -> Type {
        self.build(TypeDef::Option(type_param))
    }

    /// Builds a result type description.
    pub fn result(self, ok: TypeRef, err: TypeRef) -> Type {
        self.build(TypeDef::Result { ok, err })
    }

    /// Builds a generic parameter placeholder.
    pub fn parameter(self, name: impl Into<String>) -> Type {
        self.build(TypeDef::Parameter(name.into()))
    }

    /// Builds an applied generic type description.
    pub fn applied(self, base: TypeRef, args: Vec<TypeRef>) -> Type {
        self.build(TypeDef::Applied { base, args })
    }

    /// Starts building a composite type definition.
    pub fn composite(self) -> CompositeBuilder {
        CompositeBuilder {
            type_builder: self,
            fields: Vec::new(),
        }
    }

    /// Starts building a variant type definition.
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
