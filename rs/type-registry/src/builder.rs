use alloc::{string::String, vec::Vec};

use sails_idl_ast::{
    Annotation, EnumDef, EnumVariant, StructDef, StructField, Type, TypeDecl, TypeDef,
    TypeParameter,
};

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
    fields: Vec<StructField>,
}

/// Builder for composite (struct) type definitions.
#[derive(Debug, Clone)]
pub struct CompositeBuilder {
    type_builder: TypeBuilder,
    fields: Vec<StructField>,
}

/// Builder for variant (enum) type definitions.
#[derive(Debug, Clone)]
pub struct VariantDefBuilder {
    type_builder: TypeBuilder,
    variants: Vec<EnumVariant>,
}

/// Builder for a single declared generic parameter.
#[derive(Debug, Clone)]
pub struct ParamBuilder {
    inner: TypeBuilder,
    param_name: String,
}

/// Entry-point builder for nominal [`Type`] values.
#[derive(Debug, Clone, Default)]
pub struct TypeBuilder {
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
    fn push_field(&mut self, field: StructField);
}

impl<P: PushField> FieldBuilder<P> {
    /// Completes the field by assigning its type and returning the parent.
    pub fn ty(mut self, type_decl: TypeDecl) -> P {
        self.parent.push_field(StructField {
            name: self.name,
            type_decl,
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
        self.parent.variants.push(EnumVariant {
            name: self.name,
            def: StructDef {
                fields: self.fields,
            },
            entry_id: 0,
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

    /// Builds the final nominal struct [`Type`].
    pub fn build(self) -> Type {
        self.type_builder.build(TypeDef::Struct(StructDef {
            fields: self.fields,
        }))
    }
}

impl VariantDefBuilder {
    /// Starts a new variant inside the enum definition.
    pub fn add_variant(self, name: impl Into<String>) -> VariantBuilder {
        VariantBuilder {
            parent: self,
            name: name.into(),
            metadata: Metadata::default(),
            fields: Vec::new(),
        }
    }

    /// Builds the final nominal enum [`Type`].
    pub fn build(self) -> Type {
        self.type_builder.build(TypeDef::Enum(EnumDef {
            variants: self.variants,
        }))
    }
}

impl ParamBuilder {
    /// Assigns a default `TypeDecl` to the declared generic parameter.
    pub fn default_ty(mut self, type_decl: TypeDecl) -> TypeBuilder {
        self.inner.type_params.push(TypeParameter {
            name: self.param_name,
            ty: Some(type_decl),
        });
        self.inner
    }

    /// Completes the parameter declaration with no default.
    pub fn no_default(mut self) -> TypeBuilder {
        self.inner.type_params.push(TypeParameter {
            name: self.param_name,
            ty: None,
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

    /// Declares a generic parameter with no default.
    pub fn param(mut self, name: impl Into<String>) -> Self {
        self.type_params.push(TypeParameter {
            name: name.into(),
            ty: None,
        });
        self
    }

    /// Declares a generic parameter with a default type.
    pub fn param_with_default(mut self, name: impl Into<String>, type_decl: TypeDecl) -> Self {
        self.type_params.push(TypeParameter {
            name: name.into(),
            ty: Some(type_decl),
        });
        self
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

    /// Starts building a composite (struct) type definition.
    pub fn composite(self) -> CompositeBuilder {
        CompositeBuilder {
            type_builder: self,
            fields: Vec::new(),
        }
    }

    /// Starts building a variant (enum) type definition.
    pub fn variant(self) -> VariantDefBuilder {
        VariantDefBuilder {
            type_builder: self,
            variants: Vec::new(),
        }
    }

    /// Builds a nominal alias type definition.
    pub fn alias(self, target: TypeDecl) -> Type {
        self.build(TypeDef::Alias(sails_idl_ast::AliasDef { target }))
    }

    fn build(self, def: TypeDef) -> Type {
        Type {
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
        self.annotations.push((name.into(), None));
    }

    fn value(&mut self, value: impl Into<String>) {
        if let Some(ann) = self.annotations.last_mut() {
            ann.1 = Some(value.into());
        }
    }
}

impl PushField for CompositeBuilder {
    fn push_field(&mut self, field: StructField) {
        self.fields.push(field);
    }
}

impl PushField for VariantBuilder {
    fn push_field(&mut self, field: StructField) {
        self.fields.push(field);
    }
}
