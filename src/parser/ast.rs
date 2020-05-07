use crate::parser::span::Spanned;
use crate::parser::value::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Directive {
    pub name: Spanned<String>,
    pub arguments: Spanned<BTreeMap<Spanned<String>, Spanned<Value>>>,
}

#[derive(Clone, Debug)]
pub struct Document {
    pub definitions: Spanned<Vec<Spanned<Definition>>>,
}

#[derive(Clone, Debug)]
pub enum Definition {
    Operation(Spanned<OperationDefinition>),
    Fragment(Spanned<FragmentDefinition>),
}

#[derive(Clone, Debug)]
pub enum TypeCondition {
    On(Spanned<String>),
}

#[derive(Clone, Debug)]
pub struct FragmentDefinition {
    pub name: Spanned<String>,
    pub type_condition: Spanned<TypeCondition>,
    pub directives: Spanned<Vec<Directive>>,
    pub selection_set: Spanned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub enum OperationDefinition {
    SelectionSet(Spanned<SelectionSet>),
    Query(Spanned<Query>),
    Mutation(Spanned<Mutation>),
    Subscription(Spanned<Subscription>),
}

#[derive(Clone, Debug)]
pub struct Query {
    pub name: Option<Spanned<String>>,
    pub variable_definitions: Spanned<Vec<Spanned<VariableDefinition>>>,
    pub directives: Spanned<Vec<Directive>>,
    pub selection_set: Spanned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub struct Mutation {
    pub name: Option<Spanned<String>>,
    pub variable_definitions: Spanned<Vec<Spanned<VariableDefinition>>>,
    pub directives: Spanned<Vec<Spanned<Directive>>>,
    pub selection_set: Spanned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub struct Subscription {
    pub name: Option<Spanned<String>>,
    pub variable_definitions: Spanned<Vec<Spanned<VariableDefinition>>>,
    pub directives: Spanned<Vec<Spanned<Directive>>>,
    pub selection_set: Spanned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub struct SelectionSet {
    pub items: Spanned<Vec<Spanned<Selection>>>,
}

#[derive(Clone, Debug)]
pub struct VariableDefinition {
    pub name: Spanned<String>,
    pub var_type: Spanned<String>,
    pub default_value: Option<Spanned<Value>>,
}

#[derive(Clone, Debug)]
pub enum Selection {
    Field(Spanned<Field>),
    FragmentSpread(Spanned<FragmentSpread>),
    InlineFragment(Spanned<InlineFragment>),
}

#[derive(Clone, Debug)]
pub struct Field {
    pub alias: Option<Spanned<String>>,
    pub name: Spanned<String>,
    pub arguments: Spanned<BTreeMap<Spanned<String>, Spanned<Value>>>,
    pub directives: Spanned<Vec<Spanned<Directive>>>,
    pub selection_set: Spanned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub struct FragmentSpread {
    pub fragment_name: Spanned<String>,
    pub directives: Spanned<Vec<Spanned<Directive>>>,
}

#[derive(Clone, Debug)]
pub struct InlineFragment {
    pub type_condition: Option<Spanned<TypeCondition>>,
    pub directives: Spanned<Vec<Spanned<Directive>>>,
    pub selection_set: Spanned<SelectionSet>,
}
