use crate::parser::ast::OperationDefinition;
use crate::validation::visitor::{Visitor, VisitorContext};

#[derive(Default)]
pub struct UploadFile;

impl<'a> Visitor<'a> for UploadFile {
    fn enter_operation_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        operation_definition: &'a OperationDefinition,
    ) {
        if let OperationDefinition::Query(query) = operation_definition {
            for var in &query.variable_definitions {
                if let Some(ty) = ctx.registry.concrete_type_by_parsed_type(&var.var_type) {
                    if ty.name() == "Upload" {
                        ctx.report_error(
                            vec![var.position],
                            "The Upload type is only allowed to be defined on a mutation",
                        );
                    }
                }
            }
        } else if let OperationDefinition::Subscription(subscription) = operation_definition {
            for var in &subscription.variable_definitions {
                if let Some(ty) = ctx.registry.concrete_type_by_parsed_type(&var.var_type) {
                    if ty.name() == "Upload" {
                        ctx.report_error(
                            vec![var.position],
                            "The Upload type is only allowed to be defined on a mutation",
                        );
                    }
                }
            }
        }
    }
}
