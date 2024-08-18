use super::{FormalParameterList, FunctionBody, FunctionExpression};
use crate::{
    block_to_string,
    expression::{Expression, Identifier},
    join_nodes,
    property::{MethodDefinitionKind, PropertyName},
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
    Declaration,
};
use boa_interner::{Interner, Sym, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;
use std::hash::Hash;

/// A class declaration.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassDeclaration
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClassDeclaration {
    name: Identifier,
    super_ref: Option<Expression>,
    pub(crate) constructor: Option<FunctionExpression>,
    pub(crate) elements: Box<[ClassElement]>,
}

impl ClassDeclaration {
    /// Creates a new class declaration.
    #[inline]
    #[must_use]
    pub fn new(
        name: Identifier,
        super_ref: Option<Expression>,
        constructor: Option<FunctionExpression>,
        elements: Box<[ClassElement]>,
    ) -> Self {
        Self {
            name,
            super_ref,
            constructor,
            elements,
        }
    }

    /// Returns the name of the class declaration.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Identifier {
        self.name
    }

    /// Returns the super class ref of the class declaration.
    #[inline]
    #[must_use]
    pub const fn super_ref(&self) -> Option<&Expression> {
        self.super_ref.as_ref()
    }

    /// Returns the constructor of the class declaration.
    #[inline]
    #[must_use]
    pub const fn constructor(&self) -> Option<&FunctionExpression> {
        self.constructor.as_ref()
    }

    /// Gets the list of all fields defined on the class declaration.
    #[inline]
    #[must_use]
    pub const fn elements(&self) -> &[ClassElement] {
        &self.elements
    }
}

impl ToIndentedString for ClassDeclaration {
    fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let mut buf = format!("class {}", interner.resolve_expect(self.name.sym()));
        if let Some(super_ref) = self.super_ref.as_ref() {
            buf.push_str(&format!(
                " extends {}",
                super_ref.to_interned_string(interner)
            ));
        }
        if self.elements.is_empty() && self.constructor().is_none() {
            buf.push_str(" {}");
            return buf;
        }
        let indentation = "    ".repeat(indent_n + 1);
        buf.push_str(" {\n");
        if let Some(expr) = &self.constructor {
            buf.push_str(&format!(
                "{indentation}constructor({}) {}\n",
                join_nodes(interner, expr.parameters().as_ref()),
                block_to_string(expr.body().statements(), interner, indent_n + 1)
            ));
        }
        for element in &self.elements {
            buf.push_str(&element.to_indented_string(interner, indent_n));
        }
        buf.push('}');
        buf
    }
}

impl VisitWith for ClassDeclaration {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_identifier(&self.name));
        if let Some(expr) = &self.super_ref {
            try_break!(visitor.visit_expression(expr));
        }
        if let Some(func) = &self.constructor {
            try_break!(visitor.visit_function_expression(func));
        }
        for elem in &*self.elements {
            try_break!(visitor.visit_class_element(elem));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_identifier_mut(&mut self.name));
        if let Some(expr) = &mut self.super_ref {
            try_break!(visitor.visit_expression_mut(expr));
        }
        if let Some(func) = &mut self.constructor {
            try_break!(visitor.visit_function_expression_mut(func));
        }
        for elem in &mut *self.elements {
            try_break!(visitor.visit_class_element_mut(elem));
        }
        ControlFlow::Continue(())
    }
}

impl From<ClassDeclaration> for Declaration {
    fn from(f: ClassDeclaration) -> Self {
        Self::ClassDeclaration(f)
    }
}

/// A class expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClassExpression {
    pub(crate) name: Option<Identifier>,
    super_ref: Option<Expression>,
    pub(crate) constructor: Option<FunctionExpression>,
    pub(crate) elements: Box<[ClassElement]>,
    has_binding_identifier: bool,
}

impl ClassExpression {
    /// Creates a new class expression.
    #[inline]
    #[must_use]
    pub fn new(
        name: Option<Identifier>,
        super_ref: Option<Expression>,
        constructor: Option<FunctionExpression>,
        elements: Box<[ClassElement]>,
        has_binding_identifier: bool,
    ) -> Self {
        Self {
            name,
            super_ref,
            constructor,
            elements,
            has_binding_identifier,
        }
    }

    /// Returns the name of the class expression.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Returns the super class ref of the class expression.
    #[inline]
    #[must_use]
    pub const fn super_ref(&self) -> Option<&Expression> {
        self.super_ref.as_ref()
    }

    /// Returns the constructor of the class expression.
    #[inline]
    #[must_use]
    pub const fn constructor(&self) -> Option<&FunctionExpression> {
        self.constructor.as_ref()
    }

    /// Gets the list of all fields defined on the class expression.
    #[inline]
    #[must_use]
    pub const fn elements(&self) -> &[ClassElement] {
        &self.elements
    }
}

impl ToIndentedString for ClassExpression {
    fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let mut buf = "class".to_string();
        if self.has_binding_identifier {
            if let Some(name) = self.name {
                buf.push_str(&format!(" {}", interner.resolve_expect(name.sym())));
            }
        }
        if let Some(super_ref) = self.super_ref.as_ref() {
            buf.push_str(&format!(
                " extends {}",
                super_ref.to_interned_string(interner)
            ));
        }
        if self.elements.is_empty() && self.constructor().is_none() {
            buf.push_str(" {}");
            return buf;
        }
        let indentation = "    ".repeat(indent_n + 1);
        buf.push_str(" {\n");
        if let Some(expr) = &self.constructor {
            buf.push_str(&format!(
                "{indentation}constructor({}) {}\n",
                join_nodes(interner, expr.parameters().as_ref()),
                block_to_string(expr.body().statements(), interner, indent_n + 1)
            ));
        }
        for element in &self.elements {
            buf.push_str(&element.to_indented_string(interner, indent_n));
        }
        buf.push('}');
        buf
    }
}

impl From<ClassExpression> for Expression {
    fn from(expr: ClassExpression) -> Self {
        Self::ClassExpression(Box::new(expr))
    }
}

impl VisitWith for ClassExpression {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(ident) = &self.name {
            try_break!(visitor.visit_identifier(ident));
        }
        if let Some(expr) = &self.super_ref {
            try_break!(visitor.visit_expression(expr));
        }
        if let Some(func) = &self.constructor {
            try_break!(visitor.visit_function_expression(func));
        }
        for elem in &*self.elements {
            try_break!(visitor.visit_class_element(elem));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(ident) = &mut self.name {
            try_break!(visitor.visit_identifier_mut(ident));
        }
        if let Some(expr) = &mut self.super_ref {
            try_break!(visitor.visit_expression_mut(expr));
        }
        if let Some(func) = &mut self.constructor {
            try_break!(visitor.visit_function_expression_mut(func));
        }
        for elem in &mut *self.elements {
            try_break!(visitor.visit_class_element_mut(elem));
        }
        ControlFlow::Continue(())
    }
}

/// The body of a class' static block, as defined by the [spec].
///
/// Just an alias for [`Script`](crate::Script), since it has the same exact semantics.
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassStaticBlockBody
type StaticBlockBody = crate::Script;

/// An element that can be within a class.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassElement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum ClassElement {
    /// A method definition.
    MethodDefinition(ClassMethodDefinition),

    /// A field definition.
    FieldDefinition(PropertyName, Option<Expression>),

    /// A static field definition, accessible from the class constructor object
    StaticFieldDefinition(PropertyName, Option<Expression>),

    /// A private field definition, only accessible inside the class declaration.
    PrivateFieldDefinition(PrivateName, Option<Expression>),

    /// A private static field definition, only accessible from static methods and fields inside the
    /// class declaration.
    PrivateStaticFieldDefinition(PrivateName, Option<Expression>),

    /// A static block, where a class can have initialization logic for its static fields.
    StaticBlock(StaticBlockBody),
}

impl ToIndentedString for ClassElement {
    fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let indentation = "    ".repeat(indent_n + 1);
        match self {
            Self::MethodDefinition(m) => m.to_indented_string(interner, indent_n),
            Self::FieldDefinition(name, field) => match field {
                Some(expr) => {
                    format!(
                        "{indentation}{} = {};\n",
                        name.to_interned_string(interner),
                        expr.to_no_indent_string(interner, indent_n + 1)
                    )
                }
                None => {
                    format!("{indentation}{};\n", name.to_interned_string(interner),)
                }
            },
            Self::StaticFieldDefinition(name, field) => match field {
                Some(expr) => {
                    format!(
                        "{indentation}static {} = {};\n",
                        name.to_interned_string(interner),
                        expr.to_no_indent_string(interner, indent_n + 1)
                    )
                }
                None => {
                    format!(
                        "{indentation}static {};\n",
                        name.to_interned_string(interner),
                    )
                }
            },
            Self::PrivateFieldDefinition(name, field) => match field {
                Some(expr) => {
                    format!(
                        "{indentation}#{} = {};\n",
                        interner.resolve_expect(name.description()),
                        expr.to_no_indent_string(interner, indent_n + 1)
                    )
                }
                None => {
                    format!(
                        "{indentation}#{};\n",
                        interner.resolve_expect(name.description()),
                    )
                }
            },
            Self::PrivateStaticFieldDefinition(name, field) => match field {
                Some(expr) => {
                    format!(
                        "{indentation}static #{} = {};\n",
                        interner.resolve_expect(name.description()),
                        expr.to_no_indent_string(interner, indent_n + 1)
                    )
                }
                None => {
                    format!(
                        "{indentation}static #{};\n",
                        interner.resolve_expect(name.description()),
                    )
                }
            },
            Self::StaticBlock(body) => {
                format!(
                    "{indentation}static {}\n",
                    block_to_string(body.statements(), interner, indent_n + 1)
                )
            }
        }
    }
}

impl VisitWith for ClassElement {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Self::MethodDefinition(m) => {
                match &m.name {
                    ClassElementName::PropertyName(pn) => {
                        try_break!(visitor.visit_property_name(pn));
                    }
                    ClassElementName::PrivateName(pn) => {
                        try_break!(visitor.visit_private_name(pn));
                    }
                }
                try_break!(visitor.visit_formal_parameter_list(&m.parameters));
                visitor.visit_script(&m.body)
            }
            Self::FieldDefinition(pn, maybe_expr) | Self::StaticFieldDefinition(pn, maybe_expr) => {
                try_break!(visitor.visit_property_name(pn));
                if let Some(expr) = maybe_expr {
                    visitor.visit_expression(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            Self::PrivateFieldDefinition(name, maybe_expr)
            | Self::PrivateStaticFieldDefinition(name, maybe_expr) => {
                try_break!(visitor.visit_private_name(name));
                if let Some(expr) = maybe_expr {
                    visitor.visit_expression(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            Self::StaticBlock(sl) => visitor.visit_script(sl),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Self::MethodDefinition(m) => {
                match m.name {
                    ClassElementName::PropertyName(ref mut pn) => {
                        try_break!(visitor.visit_property_name_mut(pn));
                    }
                    ClassElementName::PrivateName(ref mut pn) => {
                        try_break!(visitor.visit_private_name_mut(pn));
                    }
                }
                try_break!(visitor.visit_formal_parameter_list_mut(&mut m.parameters));
                visitor.visit_script_mut(&mut m.body)
            }
            Self::FieldDefinition(pn, maybe_expr) | Self::StaticFieldDefinition(pn, maybe_expr) => {
                try_break!(visitor.visit_property_name_mut(pn));
                if let Some(expr) = maybe_expr {
                    visitor.visit_expression_mut(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            Self::PrivateFieldDefinition(name, maybe_expr)
            | Self::PrivateStaticFieldDefinition(name, maybe_expr) => {
                try_break!(visitor.visit_private_name_mut(name));
                if let Some(expr) = maybe_expr {
                    visitor.visit_expression_mut(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            Self::StaticBlock(sl) => visitor.visit_script_mut(sl),
        }
    }
}

/// A method definition.
///
/// This type is specific to class method definitions.
/// It includes private names and the information about whether the method is static or not.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClassMethodDefinition {
    name: ClassElementName,
    parameters: FormalParameterList,
    body: FunctionBody,
    kind: MethodDefinitionKind,
    is_static: bool,
}

impl ClassMethodDefinition {
    /// Creates a new class method definition.
    #[inline]
    #[must_use]
    pub const fn new(
        name: ClassElementName,
        parameters: FormalParameterList,
        body: FunctionBody,
        kind: MethodDefinitionKind,
        is_static: bool,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
            kind,
            is_static,
        }
    }

    /// Returns the name of the class method definition.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &ClassElementName {
        &self.name
    }

    /// Returns the parameters of the class method definition.
    #[inline]
    #[must_use]
    pub const fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Returns the body of the class method definition.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &FunctionBody {
        &self.body
    }

    /// Returns the kind of the class method definition.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> MethodDefinitionKind {
        self.kind
    }

    /// Returns whether the class method definition is static.
    #[inline]
    #[must_use]
    pub const fn is_static(&self) -> bool {
        self.is_static
    }

    /// Returns whether the class method definition is private.
    #[inline]
    #[must_use]
    pub const fn is_private(&self) -> bool {
        self.name.is_private()
    }
}

impl ToIndentedString for ClassMethodDefinition {
    fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let indentation = "    ".repeat(indent_n + 1);
        let prefix = match (self.is_static, &self.kind) {
            (true, MethodDefinitionKind::Get) => "static get ",
            (true, MethodDefinitionKind::Set) => "static set ",
            (true, MethodDefinitionKind::Ordinary) => "static ",
            (true, MethodDefinitionKind::Generator) => "static *",
            (true, MethodDefinitionKind::AsyncGenerator) => "static async *",
            (true, MethodDefinitionKind::Async) => "static async ",
            (false, MethodDefinitionKind::Get) => "get ",
            (false, MethodDefinitionKind::Set) => "set ",
            (false, MethodDefinitionKind::Ordinary) => "",
            (false, MethodDefinitionKind::Generator) => "*",
            (false, MethodDefinitionKind::AsyncGenerator) => "async *",
            (false, MethodDefinitionKind::Async) => "async ",
        };
        let name = self.name.to_interned_string(interner);
        let parameters = join_nodes(interner, self.parameters.as_ref());
        let body = block_to_string(self.body.statements(), interner, indent_n + 1);
        format!("{indentation}{prefix}{name}({parameters}) {body}\n")
    }
}

/// A class element name.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassElementName
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum ClassElementName {
    /// A property name.
    PropertyName(PropertyName),

    /// A private name.
    PrivateName(PrivateName),
}

impl ClassElementName {
    /// Returns whether the class element name is private.
    #[inline]
    #[must_use]
    pub const fn is_private(&self) -> bool {
        matches!(self, Self::PrivateName(_))
    }
}

impl ToInternedString for ClassElementName {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match &self {
            Self::PropertyName(name) => name.to_interned_string(interner),
            Self::PrivateName(name) => format!("#{}", interner.resolve_expect(name.description())),
        }
    }
}

/// A private name as defined by the [spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-private-names
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PrivateName {
    /// The `[[Description]]` internal slot of the private name.
    description: Sym,
}

impl PrivateName {
    /// Create a new private name.
    #[inline]
    #[must_use]
    pub const fn new(description: Sym) -> Self {
        Self { description }
    }

    /// Get the description of the private name.
    #[inline]
    #[must_use]
    pub const fn description(&self) -> Sym {
        self.description
    }
}

impl VisitWith for PrivateName {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_sym(&self.description)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_sym_mut(&mut self.description)
    }
}
