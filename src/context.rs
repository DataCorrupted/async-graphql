use crate::extensions::BoxExtension;
use crate::parser::ast::{Directive, Field, FragmentDefinition, SelectionSet, VariableDefinition};
use crate::registry::Registry;
use crate::{InputValueType, QueryError, Result, Schema, Type};
use crate::{Pos, Spanned, Value};
use fnv::FnvHashMap;
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

/// Variables of query
#[derive(Debug, Clone)]
pub struct Variables(Value);

impl Default for Variables {
    fn default() -> Self {
        Self(Value::Object(Default::default()))
    }
}

impl Deref for Variables {
    type Target = BTreeMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        if let Value::Object(obj) = &self.0 {
            obj
        } else {
            unreachable!()
        }
    }
}

impl DerefMut for Variables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Value::Object(obj) = &mut self.0 {
            obj
        } else {
            unreachable!()
        }
    }
}

impl Variables {
    /// Parse variables from JSON object.
    pub fn parse_from_json(value: serde_json::Value) -> Result<Self> {
        if let Value::Object(obj) = json_value_to_gql_value(value) {
            Ok(Variables(Value::Object(obj)))
        } else {
            Ok(Default::default())
        }
    }

    pub(crate) fn set_upload(
        &mut self,
        var_path: &str,
        filename: &str,
        content_type: Option<&str>,
        path: &Path,
    ) {
        let mut it = var_path.split('.').peekable();

        if let Some(first) = it.next() {
            if first != "variables" {
                return;
            }
        }

        let mut current = &mut self.0;
        while let Some(s) = it.next() {
            let has_next = it.peek().is_some();

            if let Ok(idx) = s.parse::<i32>() {
                if let Value::List(ls) = current {
                    if let Some(value) = ls.get_mut(idx as usize) {
                        if !has_next {
                            *value = Value::String(file_string(filename, content_type, path));
                            return;
                        } else {
                            current = value;
                        }
                    } else {
                        return;
                    }
                }
            } else if let Value::Object(obj) = current {
                if let Some(value) = obj.get_mut(s) {
                    if !has_next {
                        *value = Value::String(file_string(filename, content_type, path));
                        return;
                    } else {
                        current = value;
                    }
                } else {
                    return;
                }
            }
        }
    }
}

fn file_string(filename: &str, content_type: Option<&str>, path: &Path) -> String {
    if let Some(content_type) = content_type {
        format!("file:{}:{}|", filename, content_type) + &path.display().to_string()
    } else {
        format!("file:{}|", filename) + &path.display().to_string()
    }
}

fn json_value_to_gql_value(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(n) => Value::Boolean(n),
        serde_json::Value::Number(n) if n.is_f64() => Value::Float(n.as_f64().unwrap()),
        serde_json::Value::Number(n) => Value::Int((n.as_i64().unwrap() as i32).into()),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(ls) => {
            Value::List(ls.into_iter().map(json_value_to_gql_value).collect())
        }
        serde_json::Value::Object(obj) => Value::Object(
            obj.into_iter()
                .map(|(name, value)| (name, json_value_to_gql_value(value)))
                .collect(),
        ),
    }
}

#[derive(Default)]
/// Schema/Context data
pub struct Data(FnvHashMap<TypeId, Box<dyn Any + Sync + Send>>);

impl Data {
    #[allow(missing_docs)]
    pub fn insert<D: Any + Send + Sync>(&mut self, data: D) {
        self.0.insert(TypeId::of::<D>(), Box::new(data));
    }
}

/// Context for `SelectionSet`
pub type ContextSelectionSet<'a> = ContextBase<'a, &'a Spanned<SelectionSet>>;

/// Context object for resolve field
pub type Context<'a> = ContextBase<'a, &'a Spanned<Field>>;

/// The query path segment
#[derive(Clone)]
pub enum QueryPathSegment<'a> {
    /// Index
    Index(usize),

    /// Field name
    Name(&'a str),
}

/// The query path node
#[derive(Clone)]
pub struct QueryPathNode<'a> {
    /// Parent node
    pub parent: Option<&'a QueryPathNode<'a>>,

    /// Current path segment
    pub segment: QueryPathSegment<'a>,
}

impl<'a> std::fmt::Display for QueryPathNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        self.for_each(|segment| {
            if !first {
                write!(f, ".").ok();
            }
            match segment {
                QueryPathSegment::Index(idx) => {
                    write!(f, "{}", *idx).ok();
                }
                QueryPathSegment::Name(name) => {
                    write!(f, "{}", name).ok();
                }
            }
            first = false;
        });
        Ok(())
    }
}

impl<'a> QueryPathNode<'a> {
    pub(crate) fn field_name(&self) -> &str {
        let mut p = self;
        loop {
            if let QueryPathSegment::Name(name) = &p.segment {
                return name;
            }
            p = p.parent.unwrap();
        }
    }

    pub(crate) fn for_each<F: FnMut(&QueryPathSegment<'a>)>(&self, mut f: F) {
        self.for_each_ref(&mut f);
    }

    fn for_each_ref<F: FnMut(&QueryPathSegment<'a>)>(&self, f: &mut F) {
        if let Some(parent) = &self.parent {
            parent.for_each_ref(f);
        }
        f(&self.segment);
    }

    #[doc(hidden)]
    pub fn to_json(&self) -> serde_json::Value {
        let mut path: Vec<serde_json::Value> = Vec::new();
        self.for_each(|segment| {
            path.push(match segment {
                QueryPathSegment::Index(idx) => (*idx).into(),
                QueryPathSegment::Name(name) => (*name).to_string().into(),
            })
        });
        path.into()
    }
}

/// Represents the unique id of the resolve
#[derive(Copy, Clone, Debug)]
pub struct ResolveId {
    /// Parent id
    pub parent: Option<usize>,

    /// Current id
    pub current: usize,
}

impl ResolveId {
    pub(crate) fn root() -> ResolveId {
        ResolveId {
            parent: None,
            current: 0,
        }
    }
}

impl std::fmt::Display for ResolveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "{}:{}", parent, self.current)
        } else {
            write!(f, "{}", self.current)
        }
    }
}

/// Query context
#[derive(Clone)]
pub struct ContextBase<'a, T> {
    #[allow(missing_docs)]
    pub path_node: Option<QueryPathNode<'a>>,
    pub(crate) resolve_id: ResolveId,
    pub(crate) inc_resolve_id: &'a AtomicUsize,
    pub(crate) extensions: &'a [BoxExtension],
    pub(crate) item: T,
    pub(crate) variables: &'a Variables,
    pub(crate) variable_definitions: &'a [Spanned<VariableDefinition>],
    pub(crate) registry: &'a Registry,
    pub(crate) data: &'a Data,
    pub(crate) ctx_data: Option<&'a Data>,
    pub(crate) fragments: &'a HashMap<String, FragmentDefinition>,
}

impl<'a, T> Deref for ContextBase<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

#[doc(hidden)]
pub struct Environment {
    pub variables: Variables,
    pub variable_definitions: Vec<Spanned<VariableDefinition>>,
    pub fragments: HashMap<String, FragmentDefinition>,
    pub ctx_data: Arc<Data>,
}

impl Environment {
    #[doc(hidden)]
    pub fn create_context<'a, T, Query, Mutation, Subscription>(
        &'a self,
        schema: &'a Schema<Query, Mutation, Subscription>,
        path_node: Option<QueryPathNode<'a>>,
        item: T,
        inc_resolve_id: &'a AtomicUsize,
    ) -> ContextBase<'a, T> {
        ContextBase {
            path_node,
            resolve_id: ResolveId::root(),
            inc_resolve_id,
            extensions: &[],
            item,
            variables: &self.variables,
            variable_definitions: &self.variable_definitions,
            registry: &schema.0.registry,
            data: &schema.0.data,
            ctx_data: Some(&self.ctx_data),
            fragments: &self.fragments,
        }
    }
}

impl<'a, T> ContextBase<'a, T> {
    fn get_child_resolve_id(&self) -> ResolveId {
        let id = self
            .inc_resolve_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        ResolveId {
            parent: Some(self.resolve_id.current),
            current: id,
        }
    }

    #[doc(hidden)]
    pub fn with_field(&'a self, field: &'a Spanned<Field>) -> ContextBase<'a, &'a Spanned<Field>> {
        ContextBase {
            path_node: Some(QueryPathNode {
                parent: self.path_node.as_ref(),
                segment: QueryPathSegment::Name(
                    field
                        .alias
                        .map(|alias| alias.as_str())
                        .unwrap_or_else(|| field.name.as_str()),
                ),
            }),
            extensions: self.extensions,
            item: field,
            resolve_id: self.get_child_resolve_id(),
            inc_resolve_id: self.inc_resolve_id,
            variables: self.variables,
            variable_definitions: self.variable_definitions,
            registry: self.registry,
            data: self.data,
            ctx_data: self.ctx_data,
            fragments: self.fragments,
        }
    }

    #[doc(hidden)]
    pub fn with_selection_set(
        &self,
        selection_set: &'a Spanned<SelectionSet>,
    ) -> ContextBase<'a, &'a Spanned<SelectionSet>> {
        ContextBase {
            path_node: self.path_node.clone(),
            extensions: self.extensions,
            item: selection_set,
            resolve_id: self.resolve_id,
            inc_resolve_id: &self.inc_resolve_id,
            variables: self.variables,
            variable_definitions: self.variable_definitions,
            registry: self.registry,
            data: self.data,
            ctx_data: self.ctx_data,
            fragments: self.fragments,
        }
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    pub fn data<D: Any + Send + Sync>(&self) -> &D {
        self.data_opt::<D>()
            .expect("The specified data type does not exist.")
    }

    /// Gets the global data defined in the `Context` or `Schema`, returns `None` if the specified type data does not exist.
    pub fn data_opt<D: Any + Send + Sync>(&self) -> Option<&D> {
        self.ctx_data
            .and_then(|ctx_data| ctx_data.0.get(&TypeId::of::<D>()))
            .or_else(|| self.data.0.get(&TypeId::of::<D>()))
            .and_then(|d| d.downcast_ref::<D>())
    }

    fn var_value(&self, name: &str, pos: Pos) -> Result<Value> {
        let def = self
            .variable_definitions
            .iter()
            .find(|def| def.name.as_str() == name);
        if let Some(def) = def {
            if let Some(var_value) = self.variables.get(def.name.as_str()) {
                return Ok(var_value.clone());
            } else if let Some(default) = &def.default_value {
                return Ok(default.clone_inner());
            }
        }
        Err(QueryError::VarNotDefined {
            var_name: name.to_string(),
        }
        .into_error(pos))
    }

    fn resolve_input_value(&self, mut value: Value, pos: Pos) -> Result<Value> {
        match value {
            Value::Variable(var_name) => self.var_value(&var_name, pos),
            Value::List(ref mut ls) => {
                for value in ls {
                    if let Value::Variable(var_name) = value {
                        *value = self.var_value(&var_name, pos)?;
                    }
                }
                Ok(value)
            }
            Value::Object(ref mut obj) => {
                for value in obj.values_mut() {
                    if let Value::Variable(var_name) = value {
                        *value = self.var_value(&var_name, pos)?;
                    }
                }
                Ok(value)
            }
            _ => Ok(value),
        }
    }

    #[doc(hidden)]
    pub fn is_skip(&self, directives: &[Spanned<Directive>]) -> Result<bool> {
        for directive in directives {
            if directive.name.as_str() == "skip" {
                if let Some(value) = directive.arguments.get("if") {
                    let value = self.resolve_input_value(value.clone_inner(), value.position())?;
                    let res: bool = InputValueType::parse(&value).ok_or_else(|| {
                        QueryError::ExpectedType {
                            expect: bool::qualified_type_name(),
                            actual: value,
                        }
                        .into_error(directive.position())
                    })?;
                    if res {
                        return Ok(true);
                    }
                } else {
                    return Err(QueryError::RequiredDirectiveArgs {
                        directive: "@skip",
                        arg_name: "if",
                        arg_type: "Boolean!",
                    }
                    .into_error(directive.position()));
                }
            } else if directive.name.as_str() == "include" {
                if let Some(value) = directive.arguments.get("if") {
                    let value = self.resolve_input_value(value.clone_inner(), value.position())?;
                    let res: bool = InputValueType::parse(&value).ok_or_else(|| {
                        QueryError::ExpectedType {
                            expect: bool::qualified_type_name(),
                            actual: value,
                        }
                        .into_error(directive.position())
                    })?;
                    if !res {
                        return Ok(true);
                    }
                } else {
                    return Err(QueryError::RequiredDirectiveArgs {
                        directive: "@include",
                        arg_name: "if",
                        arg_type: "Boolean!",
                    }
                    .into_error(directive.position()));
                }
            } else {
                return Err(QueryError::UnknownDirective {
                    name: directive.name.clone_inner(),
                }
                .into_error(directive.position()));
            }
        }

        Ok(false)
    }
}

impl<'a> ContextBase<'a, &'a Spanned<SelectionSet>> {
    #[doc(hidden)]
    pub fn with_index(&'a self, idx: usize) -> ContextBase<'a, &'a Spanned<SelectionSet>> {
        ContextBase {
            path_node: Some(QueryPathNode {
                parent: self.path_node.as_ref(),
                segment: QueryPathSegment::Index(idx),
            }),
            extensions: self.extensions,
            item: self.item,
            resolve_id: self.get_child_resolve_id(),
            inc_resolve_id: self.inc_resolve_id,
            variables: self.variables,
            variable_definitions: self.variable_definitions,
            registry: self.registry,
            data: self.data,
            ctx_data: self.ctx_data,
            fragments: self.fragments,
        }
    }
}

impl<'a> ContextBase<'a, &'a Spanned<Field>> {
    #[doc(hidden)]
    pub fn param_value<T: InputValueType, F: FnOnce() -> Value>(
        &self,
        name: &str,
        default: F,
    ) -> Result<T> {
        match self.arguments.get(name).cloned() {
            Some(value) => {
                let pos = value.position();
                let value = self.resolve_input_value(value.into_inner(), pos)?;
                let res = InputValueType::parse(&value).ok_or_else(|| {
                    QueryError::ExpectedType {
                        expect: T::qualified_type_name(),
                        actual: value,
                    }
                    .into_error(pos)
                })?;
                Ok(res)
            }
            None => {
                let value = default();
                let res = InputValueType::parse(&value).ok_or_else(|| {
                    QueryError::ExpectedType {
                        expect: T::qualified_type_name(),
                        actual: value.clone(),
                    }
                    .into_error(self.position())
                })?;
                Ok(res)
            }
        }
    }

    #[doc(hidden)]
    pub fn result_name(&self) -> &str {
        self.item
            .alias
            .map(|alias| alias.as_str())
            .unwrap_or_else(|| self.item.name.as_str())
    }
}
