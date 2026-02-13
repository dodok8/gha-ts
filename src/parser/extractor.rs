use oxc_ast::ast::*;
use std::collections::HashSet;

pub struct ActionRefExtractor {
    pub action_refs: HashSet<String>,
}

impl ActionRefExtractor {
    pub fn new() -> Self {
        Self {
            action_refs: HashSet::new(),
        }
    }

    pub fn visit_program<'a>(&mut self, program: &Program<'a>) {
        for stmt in &program.body {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement<'a>(&mut self, stmt: &Statement<'a>) {
        match stmt {
            Statement::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    if let Some(init) = &declarator.init {
                        self.visit_expression(init);
                    }
                }
            }
            Statement::ExpressionStatement(expr_stmt) => {
                self.visit_expression(&expr_stmt.expression);
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(decl) = &export.declaration {
                    self.visit_declaration(decl);
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let Some(expr) = export.declaration.as_expression() {
                    self.visit_expression(expr);
                }
            }
            Statement::BlockStatement(block) => {
                for stmt in &block.body {
                    self.visit_statement(stmt);
                }
            }
            Statement::IfStatement(if_stmt) => {
                self.visit_expression(&if_stmt.test);
                self.visit_statement(&if_stmt.consequent);
                if let Some(alt) = &if_stmt.alternate {
                    self.visit_statement(alt);
                }
            }
            Statement::ForStatement(for_stmt) => {
                self.visit_statement(&for_stmt.body);
            }
            Statement::WhileStatement(while_stmt) => {
                self.visit_expression(&while_stmt.test);
                self.visit_statement(&while_stmt.body);
            }
            Statement::ReturnStatement(ret) => {
                if let Some(arg) = &ret.argument {
                    self.visit_expression(arg);
                }
            }
            _ => {}
        }
    }

    fn visit_declaration<'a>(&mut self, decl: &Declaration<'a>) {
        match decl {
            Declaration::VariableDeclaration(var_decl) => {
                for declarator in &var_decl.declarations {
                    if let Some(init) = &declarator.init {
                        self.visit_expression(init);
                    }
                }
            }
            Declaration::FunctionDeclaration(func) => {
                if let Some(body) = &func.body {
                    for stmt in &body.statements {
                        self.visit_statement(stmt);
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_expression<'a>(&mut self, expr: &Expression<'a>) {
        match expr {
            Expression::CallExpression(call) => {
                self.visit_call_expression(call);
            }
            Expression::NewExpression(new_expr) => {
                for arg in &new_expr.arguments {
                    match arg {
                        Argument::SpreadElement(spread) => {
                            self.visit_expression(&spread.argument);
                        }
                        _ => {
                            self.visit_expression(arg.to_expression());
                        }
                    }
                }
            }
            Expression::ArrayExpression(arr) => {
                for elem in &arr.elements {
                    match elem {
                        ArrayExpressionElement::SpreadElement(spread) => {
                            self.visit_expression(&spread.argument);
                        }
                        ArrayExpressionElement::Elision(_) => {}
                        _ => {
                            self.visit_expression(elem.to_expression());
                        }
                    }
                }
            }
            Expression::ObjectExpression(obj) => {
                for prop in &obj.properties {
                    match prop {
                        ObjectPropertyKind::ObjectProperty(p) => {
                            self.visit_expression(&p.value);
                        }
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            self.visit_expression(&spread.argument);
                        }
                    }
                }
            }
            Expression::ArrowFunctionExpression(arrow) => {
                if arrow.expression {
                    // expression body
                    for stmt in &arrow.body.statements {
                        if let Statement::ExpressionStatement(expr_stmt) = stmt {
                            self.visit_expression(&expr_stmt.expression);
                        }
                    }
                } else {
                    for stmt in &arrow.body.statements {
                        self.visit_statement(stmt);
                    }
                }
            }
            Expression::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    for stmt in &body.statements {
                        self.visit_statement(stmt);
                    }
                }
            }
            Expression::ParenthesizedExpression(paren) => {
                self.visit_expression(&paren.expression);
            }
            Expression::SequenceExpression(seq) => {
                for expr in &seq.expressions {
                    self.visit_expression(expr);
                }
            }
            Expression::ConditionalExpression(cond) => {
                self.visit_expression(&cond.test);
                self.visit_expression(&cond.consequent);
                self.visit_expression(&cond.alternate);
            }
            Expression::LogicalExpression(logical) => {
                self.visit_expression(&logical.left);
                self.visit_expression(&logical.right);
            }
            Expression::BinaryExpression(binary) => {
                self.visit_expression(&binary.left);
                self.visit_expression(&binary.right);
            }
            Expression::UnaryExpression(unary) => {
                self.visit_expression(&unary.argument);
            }
            Expression::AssignmentExpression(assign) => {
                self.visit_expression(&assign.right);
            }
            Expression::ComputedMemberExpression(computed) => {
                self.visit_expression(&computed.object);
                self.visit_expression(&computed.expression);
            }
            Expression::StaticMemberExpression(static_member) => {
                self.visit_expression(&static_member.object);
            }
            Expression::PrivateFieldExpression(private) => {
                self.visit_expression(&private.object);
            }
            Expression::ChainExpression(chain) => match &chain.expression {
                ChainElement::CallExpression(call) => {
                    self.visit_call_expression(call);
                }
                ChainElement::TSNonNullExpression(ts_non_null) => {
                    self.visit_expression(&ts_non_null.expression);
                }
                ChainElement::ComputedMemberExpression(member) => {
                    self.visit_expression(&member.object);
                }
                ChainElement::StaticMemberExpression(member) => {
                    self.visit_expression(&member.object);
                }
                ChainElement::PrivateFieldExpression(member) => {
                    self.visit_expression(&member.object);
                }
            },
            Expression::AwaitExpression(await_expr) => {
                self.visit_expression(&await_expr.argument);
            }
            Expression::YieldExpression(yield_expr) => {
                if let Some(arg) = &yield_expr.argument {
                    self.visit_expression(arg);
                }
            }
            Expression::TemplateLiteral(template) => {
                for expr in &template.expressions {
                    self.visit_expression(expr);
                }
            }
            Expression::TaggedTemplateExpression(tagged) => {
                self.visit_expression(&tagged.tag);
                for expr in &tagged.quasi.expressions {
                    self.visit_expression(expr);
                }
            }
            _ => {}
        }
    }

    fn visit_call_expression<'a>(&mut self, call: &CallExpression<'a>) {
        // Check if this is a getAction call
        match (&call.callee, call.arguments.first()) {
            (Expression::Identifier(ident), Some(Argument::StringLiteral(lit)))
                if ident.name == "getAction" =>
            {
                self.action_refs.insert(lit.value.to_string());
            }
            _ => {}
        }

        // Visit callee (for chained calls like getAction("...")(...))
        self.visit_expression(&call.callee);

        // Visit arguments
        for arg in &call.arguments {
            match arg {
                Argument::SpreadElement(spread) => {
                    self.visit_expression(&spread.argument);
                }
                _ => {
                    self.visit_expression(arg.to_expression());
                }
            }
        }
    }
}

impl Default for ActionRefExtractor {
    fn default() -> Self {
        Self::new()
    }
}
