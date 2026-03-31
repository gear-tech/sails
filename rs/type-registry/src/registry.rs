use alloc::{collections::BTreeMap, vec::Vec};
use core::{any::TypeId, fmt, num::NonZeroU32};

use crate::{
    MetaType,
    ty::{GenericArg, Type, TypeDef},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TypeRef(NonZeroU32);

impl TypeRef {
    pub fn new(id: u32) -> Self {
        Self(NonZeroU32::new(id).expect("Type ID must not be zero"))
    }

    pub fn get(&self) -> u32 {
        self.0.get()
    }
}

pub trait TypeInfo: 'static {
    type Identity: ?Sized + 'static;
    const META: MetaType = MetaType::new::<Self>();
    fn type_info(registry: &mut Registry) -> Type;
}

#[derive(Default, Debug, Clone)]
pub struct Registry {
    type_table: BTreeMap<TypeId, TypeRef>,
    types: Vec<Type>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_type<T: TypeInfo + ?Sized>(&mut self) -> TypeRef {
        self.register_meta_type(crate::meta_type::MetaType::new::<T>())
    }

    pub fn register_meta_type(&mut self, meta: crate::meta_type::MetaType) -> TypeRef {
        let type_id = meta.type_id();

        if let Some(&type_ref) = self.type_table.get(&type_id) {
            return type_ref;
        }

        let next_id = (self.types.len() as u32) + 1;
        let type_ref = TypeRef::new(next_id);

        self.type_table.insert(type_id, type_ref);
        self.types.push(Type::placeholder());

        let actual_type = meta.type_info(self);
        self.types[(next_id - 1) as usize] = actual_type;
        type_ref
    }

    pub fn register_type_def(&mut self, ty: Type) -> TypeRef {
        for (id, existing_ty) in self.types() {
            if existing_ty.name == ty.name && existing_ty.def == ty.def {
                return id;
            }
        }
        let next_id = (self.types.len() as u32) + 1;
        let type_ref = TypeRef::new(next_id);
        self.types.push(ty);
        type_ref
    }

    pub fn get_type(&self, type_ref: TypeRef) -> Option<&Type> {
        let index = (type_ref.get() as usize).checked_sub(1)?;
        self.types.get(index)
    }

    pub fn is_type<T: TypeInfo + ?Sized>(&self, type_ref: TypeRef) -> bool {
        let type_id = TypeId::of::<T::Identity>();
        self.type_table.get(&type_id) == Some(&type_ref)
    }

    pub fn types(&self) -> impl Iterator<Item = (TypeRef, &Type)> {
        self.types
            .iter()
            .enumerate()
            .map(|(i, t)| (TypeRef::new((i as u32) + 1), t))
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    pub fn display(&self, type_ref: TypeRef) -> TypeDisplay<'_> {
        TypeDisplay {
            registry: self,
            type_ref,
        }
    }
}

pub struct TypeDisplay<'a> {
    registry: &'a Registry,
    type_ref: TypeRef,
}

impl fmt::Display for TypeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(ty) = self.registry.get_type(self.type_ref) else {
            return write!(f, "<unknown>");
        };
        match &ty.def {
            TypeDef::Sequence(inner) => {
                write!(f, "[{}]", self.registry.display(*inner))
            }
            TypeDef::Array { len, type_param } => {
                write!(f, "[{}; {}]", self.registry.display(*type_param), len)
            }
            TypeDef::Tuple(elems) => {
                write!(f, "(")?;
                for (i, r) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", self.registry.display(*r))?;
                }
                write!(f, ")")
            }
            TypeDef::Map { key, value } => {
                write!(
                    f,
                    "[({}, {})]",
                    self.registry.display(*key),
                    self.registry.display(*value)
                )
            }
            TypeDef::Option(inner) => {
                write!(f, "Option<{}>", self.registry.display(*inner))
            }
            TypeDef::Result { ok, err } => {
                write!(
                    f,
                    "Result<{}, {}>",
                    self.registry.display(*ok),
                    self.registry.display(*err)
                )
            }
            TypeDef::Parameter(name) => write!(f, "{}", name),
            TypeDef::Applied { base, args } => {
                write!(f, "{}", self.registry.display(*base))?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, r) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", self.registry.display(*r))?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            TypeDef::Primitive(_) | TypeDef::Composite(_) | TypeDef::Variant(_) => {
                write!(f, "{}", ty.name)?;
                let params = &ty.type_params;
                if !params.is_empty() {
                    write!(f, "<")?;
                    for (i, p) in params.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        match &p.arg {
                            GenericArg::Type(r) => write!(f, "{}", self.registry.display(*r))?,
                            GenericArg::Const(v) => write!(f, "{}", v)?,
                        }
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            #[cfg(feature = "gprimitives")]
            TypeDef::GPrimitive(_) => write!(f, "{}", ty.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn test_type_ref_niche_optimization() {
        assert_eq!(size_of::<TypeRef>(), 4);
        assert_eq!(size_of::<Option<TypeRef>>(), 4);
        assert_eq!(size_of::<Option<u32>>(), 8);
    }

    #[test]
    fn test_type_ref_behavior() {
        let t = TypeRef::new(1);
        assert_eq!(t.get(), 1);
    }

    #[test]
    #[should_panic(expected = "Type ID must not be zero")]
    fn test_type_ref_zero_panics() {
        let _ = TypeRef::new(0);
    }

    #[test]
    fn test_registry_initial_state() {
        let registry = Registry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
}
