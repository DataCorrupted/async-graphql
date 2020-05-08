use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/query.pest"]
struct QueryParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_int() {
        assert_eq!(
            QueryParser::parse(Rule::int, "123").unwrap().as_str(),
            "123"
        );
        assert_eq!(QueryParser::parse(Rule::int, "0").unwrap().as_str(), "0");
        println!("{:?}", QueryParser::parse(Rule::int, "0123").unwrap());
    }
}
