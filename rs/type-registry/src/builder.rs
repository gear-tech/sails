use alloc::{string::String, vec::Vec};
use sails_idl_ast::{
    EnumDef, EnumVariant, NamedParam, StructDef, StructField, Type, TypeDecl, TypeDef,
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

/// Builder for composite type definitions.
#[derive(Debug, Clone)]
pub struct CompositeBuilder {
    type_builder: TypeBuilder,
    fields: Vec<StructField>,
}

/// Builder for variant type definitions.
#[derive(Debug, Clone)]
pub struct VariantDefBuilder {
    type_builder: TypeBuilder,
    variants: Vec<EnumVariant>,
}

/// Entry-point builder for synthetic [`Type`] values.
#[derive(Debug, Clone, Default)]
pub struct TypeBuilder {
    name: String,
    type_params: Vec<TypeParameter>,
    metadata: Metadata,
}

#[derive(Debug, Clone, Default)]
struct Metadata {
    docs: Vec<String>,
    annotations: Vec<(String, Option<String>)>,
}

/// Helper trait implemented by builders that can receive fields.
pub trait PushField: Sized {
    /// Appends a fully built field to the parent.
    fn push_field(&mut self, field: StructField);
}

impl<P: PushField> FieldBuilder<P> {
    /// Completes the field by assigning its type and returning the parent.
    pub fn ty(mut self, ty: TypeDecl) -> P {
        self.parent.push_field(StructField {
            name: self.name,
            type_decl: ty,
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

    /// Builds the final composite [`Type`].
    pub fn build(self) -> Type {
        self.type_builder.build(TypeDef::Struct(StructDef {
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
        self.type_builder.build(TypeDef::Enum(EnumDef {
            variants: self.variants,
        }))
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

    /// Sets the recorded module path via the @path annotation.
    pub fn module_path(mut self, module_path: impl Into<String>) -> Self {
        self.metadata.annotate(crate::PATH_ANNOTATION);
        self.metadata.value(module_path.into());
        self
    }

    /// Declares a generic parameter on the type.
    pub fn parameter(mut self, name: impl Into<String>) -> Self {
        self.type_params.push(TypeParameter {
            name: name.into(),
            ty: None,
        });
        self
    }

    /// Declares a constant generic parameter on the type.
    pub fn const_parameter(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        self.type_params.push(TypeParameter {
            name: name.into(),
            ty: Some(TypeDecl::Named {
                name: value.clone(),
                generics: Vec::new(),
                param: Some(NamedParam::Const { value }),
            }),
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
