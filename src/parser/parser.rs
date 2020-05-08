use crate::parser::ast::*;
use crate::parser::span::Spanned;
use crate::parser::value::Value;
use pest::iterators::Pair;
use pest::{error::Error, Parser};
use std::collections::BTreeMap;

#[derive(Parser)]
#[grammar = "parser/query.pest"]
struct QueryParser;

pub type ParseError = Error<Rule>;

pub fn parse_query<T: AsRef<str>>(input: T) -> Result<Document, Error<Rule>> {
    let document_pair: Pair<Rule> = QueryParser::parse(Rule::document, input.as_ref())?
        .next()
        .unwrap();
    let mut definitions = Vec::new();

    for pair in document_pair.into_inner() {
        match pair.as_rule() {
            Rule::named_operation_definition => definitions
                .push(parse_named_operation_definition(pair).pack(|op| Definition::Operation(op))),
            Rule::selection_set => definitions.push(
                parse_selection_set(pair)
                    .pack(|selection_set| OperationDefinition::SelectionSet(selection_set))
                    .pack(|operation_definition| Definition::Operation(operation_definition)),
            ),
            Rule::fragment_definition => {
                definitions.push(parse_fragment_definition(pair).pack(|f| Definition::Fragment(f)))
            }
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }
    Ok(Document { definitions })
}

fn parse_named_operation_definition(pair: Pair<Rule>) -> Spanned<OperationDefinition> {
    enum OperationType {
        Query,
        Mutation,
        Subscription,
    }

    let span = pair.as_span();
    let mut operation_type = OperationType::Query;
    let mut name = None;
    let mut variable_definitions = None;
    let mut directives = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::operation_type => {
                operation_type = match pair.as_str() {
                    "query" => OperationType::Query,
                    "mutation" => OperationType::Mutation,
                    "subscription" => OperationType::Subscription,
                    _ => unreachable!(),
                };
            }
            Rule::name => {
                name = Some(Spanned::new(pair.as_str().to_string(), pair.as_span()));
            }
            Rule::variable_definitions => {
                variable_definitions = Some(parse_variable_definitions(pair));
            }
            Rule::directives => {
                directives = Some(parse_directives(pair));
            }
            Rule::selection_set => {
                selection_set = Some(parse_selection_set(pair));
            }
            _ => unreachable!(),
        }
    }

    match operation_type {
        OperationType::Query => Spanned::new(
            Query {
                name,
                variable_definitions: variable_definitions.unwrap_or_default(),
                directives: directives.unwrap_or_default(),
                selection_set: selection_set.unwrap(),
            },
            span,
        )
        .pack(|query| OperationDefinition::Query(query)),
        OperationType::Mutation => Spanned::new(
            Mutation {
                name,
                variable_definitions: variable_definitions.unwrap_or_default(),
                directives: directives.unwrap_or_default(),
                selection_set: selection_set.unwrap(),
            },
            span,
        )
        .pack(|query| OperationDefinition::Mutation(query)),
        OperationType::Subscription => Spanned::new(
            Subscription {
                name,
                variable_definitions: variable_definitions.unwrap_or_default(),
                directives: directives.unwrap_or_default(),
                selection_set: selection_set.unwrap(),
            },
            span,
        )
        .pack(|query| OperationDefinition::Subscription(query)),
    }
}

fn parse_default_value(pair: Pair<Rule>) -> Spanned<Value> {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::value => return parse_value(pair),
            _ => unreachable!(),
        }
    }
    unreachable!()
}

fn parse_type(pair: Pair<Rule>) -> Spanned<Type> {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::nonnull_type => parse_type(pair).pack(|ty| Type::NonNull(Box::new(ty))),
        Rule::list_type => parse_type(pair).pack(|ty| Type::List(Box::new(ty))),
        Rule::name => Spanned::new(
            Type::Named(Spanned::new(pair.as_str().to_string(), pair.as_span())),
            pair.as_span(),
        ),
        Rule::type_ => parse_type(pair),
        _ => unreachable!(),
    }
}

fn parse_variable_definition(pair: Pair<Rule>) -> Spanned<VariableDefinition> {
    let span = pair.as_span();
    let mut variable = None;
    let mut ty = None;
    let mut default_value = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::variable => variable = Some(parse_variable(pair)),
            Rule::type_ => ty = Some(parse_type(pair)),
            Rule::default_value => default_value = Some(parse_default_value(pair)),
            _ => unreachable!(),
        }
    }
    Spanned::new(
        VariableDefinition {
            name: variable.unwrap(),
            var_type: ty.unwrap(),
            default_value,
        },
        span,
    )
}

fn parse_variable_definitions(pair: Pair<Rule>) -> Vec<Spanned<VariableDefinition>> {
    let mut vars = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::variable_definition => vars.push(parse_variable_definition(pair)),
            _ => unreachable!(),
        }
    }
    vars
}

fn parse_directives(pair: Pair<Rule>) -> Vec<Spanned<Directive>> {
    let directives = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::directive => {}
            _ => unreachable!(),
        }
    }
    directives
}

fn parse_variable(pair: Pair<Rule>) -> Spanned<String> {
    for pair in pair.into_inner() {
        if let Rule::name = pair.as_rule() {
            return Spanned::new(pair.as_str().to_string(), pair.as_span());
        }
    }
    unreachable!()
}

fn parse_value(pair: Pair<Rule>) -> Spanned<Value> {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::object => parse_object_value(pair),
        Rule::array => parse_array_value(pair),
        Rule::variable => parse_variable(pair).map(|var| Value::Variable(var)),
        Rule::float => Spanned::new(Value::Float(pair.as_str().parse().unwrap()), pair.as_span()),
        Rule::int => Spanned::new(Value::Int(pair.as_str().parse().unwrap()), pair.as_span()),
        Rule::string => Spanned::new(Value::String(pair.as_str().to_string()), pair.as_span()),
        Rule::name => Spanned::new(Value::Enum(pair.as_str().to_string()), pair.as_span()),
        Rule::boolean => Spanned::new(
            Value::Boolean(match pair.as_str() {
                "true" => true,
                "false" => false,
                _ => unreachable!(),
            }),
            pair.as_span(),
        ),
        Rule::null => Spanned::new(Value::Null, pair.as_span()),
        _ => unreachable!(),
    }
}

fn parse_object_value(pair: Pair<Rule>) -> Spanned<Value> {
    let span = pair.as_span();
    let mut map = BTreeMap::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::pair => {
                map.extend(std::iter::once(parse_pair(pair)));
            }
            _ => unreachable!(),
        }
    }
    Spanned::new(Value::Object(map), span)
}

fn parse_array_value(pair: Pair<Rule>) -> Spanned<Value> {
    let span = pair.as_span();
    let mut array = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::value => {
                array.push(parse_value(pair));
            }
            _ => unreachable!(),
        }
    }
    Spanned::new(Value::List(array), span)
}

fn parse_pair(pair: Pair<Rule>) -> (Spanned<String>, Spanned<Value>) {
    let mut name = None;
    let mut value = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(Spanned::new(pair.as_str().to_string(), pair.as_span())),
            Rule::value => value = Some(parse_value(pair)),
            _ => unreachable!(),
        }
    }
    (name.unwrap(), value.unwrap())
}

fn parse_arguments(pair: Pair<Rule>) -> BTreeMap<Spanned<String>, Spanned<Value>> {
    let mut arguments = BTreeMap::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::pair => arguments.extend(std::iter::once(parse_pair(pair))),
            _ => unreachable!(),
        }
    }
    arguments
}

fn parse_alias(pair: Pair<Rule>) -> Spanned<String> {
    for pair in pair.into_inner() {
        if let Rule::name = pair.as_rule() {
            return Spanned::new(pair.as_str().to_string(), pair.as_span());
        }
    }
    unreachable!()
}

fn parse_field(pair: Pair<Rule>) -> Spanned<Field> {
    let span = pair.as_span();
    let mut alias = None;
    let mut name = None;
    let mut directives = None;
    let mut arguments = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::alias => alias = Some(parse_alias(pair)),
            Rule::name => name = Some(Spanned::new(pair.as_str().to_string(), pair.as_span())),
            Rule::arguments => arguments = Some(parse_arguments(pair)),
            Rule::directives => directives = Some(parse_directives(pair)),
            Rule::selection_set => selection_set = Some(parse_selection_set(pair)),
            _ => unreachable!(),
        }
    }

    Spanned::new(
        Field {
            alias,
            name: name.unwrap(),
            arguments: arguments.unwrap_or_default(),
            directives: directives.unwrap_or_default(),
            selection_set: selection_set.unwrap_or_default(),
        },
        span,
    )
}

fn parse_fragment_spread(pair: Pair<Rule>) -> Spanned<FragmentSpread> {
    let span = pair.as_span();
    let mut name = None;
    let mut directives = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(Spanned::new(pair.as_str().to_string(), pair.as_span())),
            Rule::directives => directives = Some(parse_directives(pair)),
            _ => unreachable!(),
        }
    }
    Spanned::new(
        FragmentSpread {
            fragment_name: name.unwrap(),
            directives: directives.unwrap_or_default(),
        },
        span,
    )
}

fn parse_type_condition(pair: Pair<Rule>) -> Spanned<TypeCondition> {
    for pair in pair.into_inner() {
        if let Rule::name = pair.as_rule() {
            return Spanned::new(
                TypeCondition::On(Spanned::new(pair.as_str().to_string(), pair.as_span())),
                pair.as_span(),
            );
        }
    }
    unreachable!()
}

fn parse_inline_fragment(pair: Pair<Rule>) -> Spanned<InlineFragment> {
    let span = pair.as_span();
    let mut type_condition = None;
    let mut directives = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::type_condition => type_condition = Some(parse_type_condition(pair)),
            Rule::directives => directives = Some(parse_directives(pair)),
            Rule::selection_set => selection_set = Some(parse_selection_set(pair)),
            _ => unreachable!(),
        }
    }

    Spanned::new(
        InlineFragment {
            type_condition,
            directives: directives.unwrap_or_default(),
            selection_set: selection_set.unwrap(),
        },
        span,
    )
}

fn parse_selection_set(pair: Pair<Rule>) -> Spanned<SelectionSet> {
    let span = pair.as_span();
    let mut items = Vec::new();
    for pair in pair.into_inner().map(|pair| pair.into_inner()).flatten() {
        match pair.as_rule() {
            Rule::field => items.push(parse_field(pair).pack(|field| Selection::Field(field))),
            Rule::fragment_spread => {
                items.push(parse_fragment_spread(pair).pack(|f| Selection::FragmentSpread(f)))
            }
            Rule::inline_fragment => {
                items.push(parse_inline_fragment(pair).pack(|f| Selection::InlineFragment(f)))
            }
            _ => unreachable!(),
        }
    }
    Spanned::new(SelectionSet { items }, span)
}

fn parse_fragment_definition(pair: Pair<Rule>) -> Spanned<FragmentDefinition> {
    let span = pair.as_span();
    let mut name = None;
    let mut type_condition = None;
    let mut directives = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(Spanned::new(pair.as_str().to_string(), pair.as_span())),
            Rule::type_condition => type_condition = Some(parse_type_condition(pair)),
            Rule::directives => directives = Some(parse_directives(pair)),
            Rule::selection_set => selection_set = Some(parse_selection_set(pair)),
            _ => unreachable!(),
        }
    }

    Spanned::new(
        FragmentDefinition {
            name: name.unwrap(),
            type_condition: type_condition.unwrap(),
            directives: directives.unwrap_or_default(),
            selection_set: selection_set.unwrap(),
        },
        span,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parser() {
        for entry in fs::read_dir("tests/queries").unwrap() {
            if let Ok(entry) = entry {
                QueryParser::parse(Rule::document, &fs::read_to_string(entry.path()).unwrap())
                    .unwrap();
            }
        }
    }

    #[test]
    fn test_parser_ast() {
        for entry in fs::read_dir("tests/queries").unwrap() {
            if let Ok(entry) = entry {
                parse_query(fs::read_to_string(entry.path()).unwrap()).unwrap();
            }
        }
    }
}
