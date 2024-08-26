use rustpython_parser::ast;
use rustpython_parser::ast::Stmt;
use rustpython_parser::ast::Stmt::FunctionDef;

fn generate_parameter_ids(test_name: &str, template: &str, count: usize) -> Vec<String> {
    // Split the template into variable names
    let variables: Vec<&str> = template.split(',').map(|s| s.trim()).collect();

    let mut result = Vec::new();

    // Generate combinations for each index from 0 to count-1
    for i in 0..count {
        let mut combination = Vec::new();
        for var in &variables {
            combination.push(format!("{}{}", var, i));
        }
        result.push(format!("{}[{}]", test_name, combination.join("-")));
    }

    result
}

pub fn expand_parameters(stmt: Stmt) -> Option<Vec<String>> {
    let mut parameterizations = vec![];

    match stmt {
        FunctionDef(node) => {
            for decorator in node.decorator_list {
                if let ast::Expr::Call(call) = decorator {
                    if let Some(attr_expr) = call.func.as_attribute_expr() {
                        if let Some(nested_attr_expr) = attr_expr.value.as_attribute_expr() {
                            if let Some(name_expr) = nested_attr_expr.value.as_name_expr() {
                                let module = name_expr.id.as_str();
                                let is_parametrized = module == "pytest"
                                    && nested_attr_expr.attr.as_str() == "mark"
                                    && attr_expr.attr.as_str() == "parametrize";

                                if is_parametrized {
                                    let const_expr =
                                        call.args[0].clone().constant_expr().unwrap().clone();
                                    let parameters = const_expr.value.as_str().unwrap();
                                    let values = call.args[1].clone();
                                    match values {
                                        ast::Expr::Tuple(tuple) => {
                                            parameterizations.extend(generate_parameter_ids(
                                                node.name.as_str(),
                                                parameters,
                                                tuple.elts.len(),
                                            ))
                                        }

                                        ast::Expr::List(list) => {
                                            parameterizations.extend(generate_parameter_ids(
                                                node.name.as_str(),
                                                parameters,
                                                list.elts.len(),
                                            ))
                                        }

                                        _ => println!("Unsupported parameterization {:#?}", values),
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !parameterizations.is_empty() {
                Some(parameterizations)
            } else {
                Some(vec![node.name.to_string()])
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::indoc::indoc;
    use rustpython_parser::{ast, Parse};

    #[test]
    fn it_works_with_non_parameterized_test() {
        let code = indoc! {"
            def test_not_parameterized():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = expand_parameters(ast.unwrap().first().take().unwrap().clone());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec!["test_not_parameterized"]);
    }

    #[test]
    fn it_works_with_single_parameterized_test() {
        let code = indoc! {"
            @pytest.mark.parametrize('a', [1, 2, 3])
            def test_parameterized():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = expand_parameters(ast.unwrap().first().take().unwrap().clone());
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            vec![
                "test_parameterized[a0]",
                "test_parameterized[a1]",
                "test_parameterized[a2]"
            ]
        );
    }

    #[test]
    fn it_works_with_single_parameterized_test_multiple_args() {
        let code = indoc! {"
            @pytest.mark.parametrize('a, b', [(1, 2), (2, 3), (3, 4)])
            def test_parameterized():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = expand_parameters(ast.unwrap().first().take().unwrap().clone());
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            vec![
                "test_parameterized[a0-b0]",
                "test_parameterized[a1-b1]",
                "test_parameterized[a2-b2]"
            ]
        );
    }

    #[test]
    fn it_works_with_single_parameterized_test_multiple_args_tuple() {
        let code = indoc! {"
            @pytest.mark.parametrize('a, b', ((1, 2), (2, 3), (3, 4)))
            def test_parameterized():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = expand_parameters(ast.unwrap().first().take().unwrap().clone());
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            vec![
                "test_parameterized[a0-b0]",
                "test_parameterized[a1-b1]",
                "test_parameterized[a2-b2]"
            ]
        );
    }

    #[test]
    fn it_doesnt_blow_up_on_weird_stuff() {
        let code = indoc! {"
            @pytest.mark.parametrize('a', [foo for foo in range(10)])
            def test_parameterized(a):
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = expand_parameters(ast.unwrap().first().take().unwrap().clone());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec!["test_parameterized",]);
    }
}
