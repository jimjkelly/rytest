use rustpython_parser::ast;
use rustpython_parser::ast::Stmt;
use rustpython_parser::ast::Stmt::FunctionDef;

pub fn is_parametrized(stmt: &Stmt) -> bool {
    match stmt {
        FunctionDef(node) => {
            for decorator in &node.decorator_list {
                if let ast::Expr::Call(call) = decorator {
                    if let Some(attr_expr) = call.func.as_attribute_expr() {
                        if let Some(nested_attr_expr) = attr_expr.value.as_attribute_expr() {
                            if let Some(name_expr) = nested_attr_expr.value.as_name_expr() {
                                let module = name_expr.id.as_str();
                                if module == "pytest"
                                    && nested_attr_expr.attr.as_str() == "mark"
                                    && attr_expr.attr.as_str() == "parametrize"
                                {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            false
        }
        _ => false,
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
        let result = is_parametrized(ast.unwrap().first().take().unwrap());
        assert!(!result);
    }

    #[test]
    fn it_works_with_single_parameterized_test() {
        let code = indoc! {"
            @pytest.mark.parametrize('a', [1, 2, 3])
            def test_parameterized():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = is_parametrized(ast.unwrap().first().take().unwrap());
        assert!(result);
    }

    #[test]
    fn it_works_with_single_parameterized_test_multiple_args() {
        let code = indoc! {"
            @pytest.mark.parametrize('a, b', [(1, 2), (2, 3), (3, 4)])
            def test_parameterized():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = is_parametrized(ast.unwrap().first().take().unwrap());
        assert!(result);
    }

    #[test]
    fn it_works_with_single_parameterized_test_multiple_args_tuple() {
        let code = indoc! {"
            @pytest.mark.parametrize('a, b', ((1, 2), (2, 3), (3, 4)))
            def test_parameterized():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = is_parametrized(ast.unwrap().first().take().unwrap());
        assert!(result);
    }

    #[test]
    fn it_doesnt_blow_up_on_weird_stuff() {
        let code = indoc! {"
            @pytest.mark.parametrize('a', [foo for foo in range(10)])
            def test_parameterized(a):
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = is_parametrized(ast.unwrap().first().take().unwrap());
        assert!(result);
    }
}
