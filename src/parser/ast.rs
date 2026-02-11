use oxc_ast::ast::*;
use std::collections::HashSet;

pub struct ActionRefVisitor<'a> {
    pub action_refs: HashSet<String>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> ActionRefVisitor<'a> {
    pub fn new() -> Self {
        Self {
            action_refs: HashSet::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn is_get_action_call(&self, expr: &CallExpression<'a>) -> bool {
        match &expr.callee {
            Expression::Identifier(ident) => ident.name == "getAction",
            _ => false,
        }
    }

    pub fn extract_string_literal(&mut self, arg: &Argument<'a>) {
        if let Argument::StringLiteral(lit) = arg {
            self.action_refs.insert(lit.value.to_string());
        }
    }
}

impl Default for ActionRefVisitor<'_> {
    fn default() -> Self {
        Self::new()
    }
}
