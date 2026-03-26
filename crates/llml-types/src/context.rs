//! Type checking context — manages type environment during checking.

use std::collections::HashMap;

use crate::errors::{TypeErrorKind, TypeErrors};
use crate::types::{FnType, Type, TypeVarId, resolve_named_type};
use llml_lexer::Span;
use llml_parser::ast::*;

/// A binding in the type environment.
#[derive(Debug, Clone)]
pub struct TypeBinding {
    pub ty: Type,
    pub mutable: bool,
}

/// Variant of a sum type.
#[derive(Debug, Clone)]
pub struct VariantDef {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

/// Definition of a user-defined type (sum or product).
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<VariantDef>,
    pub is_sum: bool,
}

/// Type checking context — tracks variables, types, and generic bindings.
pub struct TypeContext {
    /// Variable scopes: name -> (type, mutable)
    scopes: Vec<HashMap<String, TypeBinding>>,
    /// User-defined type definitions: type name -> TypeDef
    type_defs: HashMap<String, TypeDef>,
    /// Constructor -> (owning type name, field types)
    constructors: HashMap<String, (String, Vec<Type>)>,
    /// Accumulated errors
    pub errors: TypeErrors,
    /// Next fresh type variable id
    next_var: TypeVarId,
    /// Substitution map for type variables
    substitutions: HashMap<TypeVarId, Type>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            type_defs: HashMap::new(),
            constructors: HashMap::new(),
            errors: TypeErrors::new(),
            next_var: 0,
            substitutions: HashMap::new(),
        }
    }

    // ── Scope management ──

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a variable binding in the current scope.
    pub fn define_var(&mut self, name: String, ty: Type, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, TypeBinding { ty, mutable });
        }
    }

    /// Look up a variable's type.
    pub fn get_var(&self, name: &str) -> Option<&TypeBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(name) {
                return Some(binding);
            }
        }
        None
    }

    // ── Type definitions ──

    /// Register a user-defined type.
    pub fn define_type(&mut self, def: TypeDef) {
        for variant in &def.variants {
            let field_types: Vec<Type> = variant.fields.iter().map(|(_, t)| t.clone()).collect();
            self.constructors
                .insert(variant.name.clone(), (def.name.clone(), field_types));
        }
        self.type_defs.insert(def.name.clone(), def);
    }

    /// Look up a type definition.
    pub fn get_type_def(&self, name: &str) -> Option<&TypeDef> {
        self.type_defs.get(name)
    }

    /// Look up a constructor's owning type and field types.
    pub fn get_constructor(&self, name: &str) -> Option<&(String, Vec<Type>)> {
        self.constructors.get(name)
    }

    // ── Type variables / unification ──

    /// Create a fresh type variable.
    pub fn fresh_var(&mut self) -> Type {
        let id = self.next_var;
        self.next_var += 1;
        Type::Var(id)
    }

    /// Apply substitutions to resolve type variables.
    pub fn resolve(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(id) => {
                if let Some(resolved) = self.substitutions.get(id) {
                    self.resolve(resolved)
                } else {
                    ty.clone()
                }
            }
            Type::Fn(ft) => Type::Fn(FnType {
                params: ft.params.iter().map(|p| self.resolve(p)).collect(),
                ret: Box::new(self.resolve(&ft.ret)),
                effects: ft.effects.clone(),
            }),
            Type::Adt(name, args) => {
                Type::Adt(name.clone(), args.iter().map(|a| self.resolve(a)).collect())
            }
            Type::Linear(inner) => Type::Linear(Box::new(self.resolve(inner))),
            Type::Ref(inner) => Type::Ref(Box::new(self.resolve(inner))),
            _ => ty.clone(),
        }
    }

    /// Unify two types, recording substitutions.
    pub fn unify(&mut self, a: &Type, b: &Type, span: Span) -> bool {
        let a = self.resolve(a);
        let b = self.resolve(b);

        if a == b {
            return true;
        }

        match (&a, &b) {
            (Type::Error, _) | (_, Type::Error) => true,
            (Type::Var(id), _) => {
                self.substitutions.insert(*id, b);
                true
            }
            (_, Type::Var(id)) => {
                self.substitutions.insert(*id, a);
                true
            }
            (Type::Fn(ft1), Type::Fn(ft2)) => {
                if ft1.params.len() != ft2.params.len() {
                    self.errors.push(
                        TypeErrorKind::UnificationFailure(a.clone(), b.clone()),
                        span,
                    );
                    return false;
                }
                let mut ok = true;
                for (p1, p2) in ft1.params.iter().zip(ft2.params.iter()) {
                    if !self.unify(p1, p2, span) {
                        ok = false;
                    }
                }
                if !self.unify(&ft1.ret, &ft2.ret, span) {
                    ok = false;
                }
                ok
            }
            (Type::Adt(n1, a1), Type::Adt(n2, a2)) if n1 == n2 && a1.len() == a2.len() => {
                let mut ok = true;
                for (t1, t2) in a1.iter().zip(a2.iter()) {
                    if !self.unify(t1, t2, span) {
                        ok = false;
                    }
                }
                ok
            }
            (Type::Linear(i1), Type::Linear(i2)) => self.unify(i1, i2, span),
            (Type::Ref(i1), Type::Ref(i2)) => self.unify(i1, i2, span),
            _ => {
                self.errors.push(
                    TypeErrorKind::Mismatch {
                        expected: a,
                        found: b,
                    },
                    span,
                );
                false
            }
        }
    }

    // ── AST type resolution ──

    /// Convert an AST TypeExpr to our internal Type representation.
    pub fn resolve_type_expr(
        &mut self,
        texpr: &TypeExpr,
        generics: &HashMap<String, Type>,
    ) -> Type {
        match texpr {
            TypeExpr::Named(name) => {
                if let Some(ty) = resolve_named_type(name) {
                    ty
                } else {
                    Type::Adt(name.clone(), vec![])
                }
            }
            TypeExpr::Generic(name) => {
                if let Some(ty) = generics.get(name) {
                    ty.clone()
                } else {
                    self.fresh_var()
                }
            }
            TypeExpr::App(name, args) => {
                let resolved_args: Vec<Type> = args
                    .iter()
                    .map(|a| self.resolve_type_expr(&a.inner, generics))
                    .collect();
                Type::Adt(name.clone(), resolved_args)
            }
            TypeExpr::FnType(params, ret) => {
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|p| self.resolve_type_expr(&p.inner, generics))
                    .collect();
                let ret_type = self.resolve_type_expr(&ret.inner, generics);
                Type::Fn(FnType {
                    params: param_types,
                    ret: Box::new(ret_type),
                    effects: vec![],
                })
            }
            TypeExpr::Linear(inner) => {
                Type::Linear(Box::new(self.resolve_type_expr(&inner.inner, generics)))
            }
            TypeExpr::Ref(inner) => {
                Type::Ref(Box::new(self.resolve_type_expr(&inner.inner, generics)))
            }
        }
    }

    /// Report an error.
    pub fn error(&mut self, kind: TypeErrorKind, span: Span) {
        self.errors.push(kind, span);
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}
