use crate::parser::ast::Field;
use crate::validation::visitor::{Visitor, VisitorContext};

pub struct ComplexityCalculate<'a> {
    pub complexity: &'a mut usize,
}

impl<'ctx, 'a> Visitor<'ctx> for ComplexityCalculate<'a> {
    fn enter_field(&mut self, _ctx: &mut VisitorContext<'_>, _field: &Field) {
        *self.complexity += 1;
    }
}
