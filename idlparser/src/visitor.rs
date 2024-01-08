use crate::types::*;

pub trait Visitor<'ast> {
    fn visit_program(&mut self, program: &'ast Program) {
        program.accept(self);
    }

    fn visit_service(&mut self, service: &'ast Service) {
        service.accept(self);
    }

    fn visit_type(&mut self, r#type: &'ast Type) {
        r#type.accept(self);
    }

    fn visit_optional_type_decl(&mut self, optional_type_decl: &'ast TypeDecl) {
        optional_type_decl.accept(self);
    }

    fn visit_vector_type_decl(&mut self, vector_type_decl: &'ast TypeDecl) {
        vector_type_decl.accept(self);
    }

    fn visit_result_type_decl(
        &mut self,
        ok_type_decl: &'ast TypeDecl,
        err_type_decl: &'ast TypeDecl,
    ) {
        ok_type_decl.accept(self);
        err_type_decl.accept(self);
    }

    fn visit_primitive_type_id(&mut self, _primitive_type_id: &'ast PrimitiveType) {}

    fn visit_user_defined_type_id(&mut self, _user_defined_type_id: &'ast str) {}

    fn visit_func(&mut self, func: &'ast Func) {
        func.accept(self)
    }

    fn visit_param(&mut self, param: &'ast Param) {
        param.accept(self);
    }

    fn visit_output(&mut self, output: &'ast TypeDecl) {
        output.accept(self);
    }

    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        struct_def.accept(self);
    }

    fn visit_struct_field(&mut self, struct_field: &'ast StructField) {
        struct_field.accept(self);
    }

    fn visit_enum_def(&mut self, enum_def: &'ast EnumDef) {
        enum_def.accept(self);
    }

    fn visit_enum_variant(&mut self, enum_variant: &'ast EnumVariant) {
        enum_variant.accept(self);
    }
}
