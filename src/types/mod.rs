pub mod object;
use oxc::{
    ast::{
        ast::{CallExpression, Declaration, Expression, Function, Statement},
        AstKind,
    },
    semantic::AstNode,
};

trait Traverse {
    fn traverse(&self);
}

#[allow(unused)]
impl<'a> Traverse for Statement<'a> {
    fn traverse(&self) {
        match &self {
            Statement::BlockStatement(statement) => {}
            Statement::BreakStatement(statement) => {}
            Statement::ContinueStatement(statement) => {}
            Statement::DebuggerStatement(statement) => {}
            Statement::DoWhileStatement(statement) => {}
            Statement::EmptyStatement(statement) => {}
            Statement::ExpressionStatement(statement) => {
                statement.expression.traverse();
            }
            Statement::ForInStatement(statement) => {}
            Statement::ForOfStatement(statement) => {}
            Statement::ForStatement(statement) => {}
            Statement::IfStatement(statement) => {}
            Statement::LabeledStatement(statement) => {}
            Statement::ReturnStatement(statement) => {}
            Statement::SwitchStatement(statement) => {}
            Statement::ThrowStatement(statement) => {}
            Statement::TryStatement(statement) => {}
            Statement::WhileStatement(statement) => {}
            Statement::WithStatement(statement) => {}
            Statement::ModuleDeclaration(statement) => {}
            Statement::Declaration(statement) => {
                statement.traverse();
            }
        }
    }
}

#[allow(unused)]
impl<'a> Traverse for Declaration<'a> {
    fn traverse(&self) {
        match &self {
            Declaration::VariableDeclaration(declaration) => {}
            Declaration::FunctionDeclaration(declaration) => {
                println!("{:#?}", declaration.id);
                println!("{:#?}", declaration.params);
            }
            Declaration::ClassDeclaration(declaration) => {}
            Declaration::TSTypeAliasDeclaration(declaration) => {}
            Declaration::TSInterfaceDeclaration(declaration) => {}
            Declaration::TSEnumDeclaration(declaration) => {}
            Declaration::TSModuleDeclaration(declaration) => {}
            Declaration::TSImportEqualsDeclaration(declaration) => {}
        }
    }
}

#[allow(unused)]
impl<'a> Traverse for CallExpression<'a> {
    fn traverse(&self) {
        println!("Args: {:#?}", self.arguments);
    }
}

#[allow(unused)]
impl<'a> Traverse for Expression<'a> {
    fn traverse(&self) {
        match &self {
            Expression::BooleanLiteral(expr) => {}
            Expression::NullLiteral(expr) => {}
            Expression::NumberLiteral(expr) => {}
            Expression::BigintLiteral(expr) => {}
            Expression::RegExpLiteral(expr) => {}
            Expression::StringLiteral(expr) => {}
            Expression::TemplateLiteral(expr) => {}

            Expression::Identifier(expr) => {}

            Expression::MetaProperty(expr) => {}
            Expression::Super(expr) => {}

            Expression::ArrayExpression(expr) => {}
            Expression::ArrowExpression(expr) => {}
            Expression::AssignmentExpression(expr) => {}
            Expression::AwaitExpression(expr) => {}
            Expression::BinaryExpression(expr) => {}
            Expression::CallExpression(expr) => {
                expr.traverse();
            }
            Expression::ChainExpression(expr) => {}
            Expression::ClassExpression(expr) => {}
            Expression::ConditionalExpression(expr) => {}
            Expression::FunctionExpression(expr) => {}
            Expression::ImportExpression(expr) => {}
            Expression::LogicalExpression(expr) => {}
            Expression::MemberExpression(expr) => {}
            Expression::NewExpression(expr) => {}
            Expression::ObjectExpression(expr) => {}
            Expression::ParenthesizedExpression(expr) => {}
            Expression::SequenceExpression(expr) => {}
            Expression::TaggedTemplateExpression(expr) => {}
            Expression::ThisExpression(expr) => {}
            Expression::UnaryExpression(expr) => {}
            Expression::UpdateExpression(expr) => {}
            Expression::YieldExpression(expr) => {}
            Expression::PrivateInExpression(expr) => {}

            Expression::JSXElement(expr) => {}
            Expression::JSXFragment(expr) => {}

            Expression::TSAsExpression(expr) => {}
            Expression::TSSatisfiesExpression(expr) => {}
            Expression::TSTypeAssertion(expr) => {}
            Expression::TSNonNullExpression(expr) => {}
            Expression::TSInstantiationExpression(expr) => {}
        }
    }
}

impl<'a> Traverse for Function<'a> {
    fn traverse(&self) {
        println!("{:#?}", self.params.items);

        // extract what are the properties we are referring to, for the argument
        if let Some(body) = &self.body {
            for item in body.statements.iter() {
                item.traverse();
            }
        }
    }
}

#[allow(unused)]
impl<'a> Traverse for AstKind<'a> {
    fn traverse(&self) {
        match &self {
            AstKind::Program(item) => {}
            AstKind::Directive(item) => {}
            AstKind::Hashbang(item) => {}

            AstKind::BlockStatement(item) => {}
            AstKind::BreakStatement(item) => {}
            AstKind::ContinueStatement(item) => {}
            AstKind::DebuggerStatement(item) => {}
            AstKind::DoWhileStatement(item) => {}
            AstKind::EmptyStatement(item) => {}
            AstKind::ExpressionStatement(item) => {}
            AstKind::ForInStatement(item) => {}
            AstKind::ForOfStatement(item) => {}
            AstKind::ForStatement(item) => {}
            AstKind::ForStatementInit(item) => {}
            AstKind::IfStatement(item) => {}
            AstKind::LabeledStatement(item) => {}
            AstKind::ReturnStatement(item) => {}
            AstKind::SwitchStatement(item) => {}
            AstKind::ThrowStatement(item) => {}
            AstKind::TryStatement(item) => {}
            AstKind::WhileStatement(item) => {}
            AstKind::WithStatement(item) => {}

            AstKind::SwitchCase(item) => {}
            AstKind::CatchClause(item) => {}
            AstKind::FinallyClause(item) => {}

            AstKind::VariableDeclaration(item) => {}
            AstKind::VariableDeclarator(item) => {}

            AstKind::IdentifierName(item) => {}
            AstKind::IdentifierReference(item) => {}
            AstKind::BindingIdentifier(item) => {}
            AstKind::LabelIdentifier(item) => {}
            AstKind::PrivateIdentifier(item) => {}

            AstKind::NumberLiteral(item) => {}
            AstKind::StringLiteral(item) => {}
            AstKind::BooleanLiteral(item) => {}
            AstKind::NullLiteral(item) => {}
            AstKind::BigintLiteral(item) => {}
            AstKind::RegExpLiteral(item) => {}
            AstKind::TemplateLiteral(item) => {}

            AstKind::MetaProperty(item) => {}
            AstKind::Super(item) => {}

            AstKind::ArrayExpression(item) => {}
            AstKind::ArrowExpression(item) => {}
            AstKind::AssignmentExpression(item) => {}
            AstKind::AwaitExpression(item) => {}
            AstKind::BinaryExpression(item) => {}
            AstKind::CallExpression(item) => {}
            AstKind::ConditionalExpression(item) => {}
            AstKind::LogicalExpression(item) => {}
            AstKind::MemberExpression(item) => {}
            AstKind::NewExpression(item) => {}
            AstKind::ObjectExpression(item) => {}
            AstKind::ParenthesizedExpression(item) => {}
            AstKind::SequenceExpression(item) => {}
            AstKind::TaggedTemplateExpression(item) => {}
            AstKind::ThisExpression(item) => {}
            AstKind::UnaryExpression(item) => {}
            AstKind::UpdateExpression(item) => {}
            AstKind::YieldExpression(item) => {}

            AstKind::ObjectProperty(item) => {}
            AstKind::PropertyKey(item) => {}
            AstKind::Argument(item) => {}
            AstKind::AssignmentTarget(item) => {}
            AstKind::SimpleAssignmentTarget(item) => {}
            AstKind::AssignmentTargetWithDefault(item) => {}
            AstKind::ArrayExpressionElement(item) => {}
            AstKind::Elision(span) => {}
            AstKind::SpreadElement(item) => {}
            AstKind::RestElement(item) => {}

            AstKind::Function(item) => {
                item.traverse();
            }
            AstKind::FunctionBody(item) => {}
            AstKind::FormalParameters(item) => {}
            AstKind::FormalParameter(item) => {}

            AstKind::Class(item) => {}
            AstKind::ClassHeritage(item) => {}
            AstKind::StaticBlock(item) => {}
            AstKind::PropertyDefinition(item) => {}
            AstKind::MethodDefinition(item) => {}

            AstKind::ArrayPattern(item) => {}
            AstKind::ObjectPattern(item) => {}
            AstKind::AssignmentPattern(item) => {}

            AstKind::Decorator(item) => {}

            AstKind::ModuleDeclaration(item) => {}
            _ => {}
        }
    }
}

impl<'a> Traverse for AstNode<'a> {
    fn traverse(&self) {
        self.kind().traverse();
    }
}
