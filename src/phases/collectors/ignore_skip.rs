use rustpython_parser::ast;
use rustpython_parser::ast::Stmt;
use rustpython_parser::ast::Stmt::FunctionDef;

pub fn is_pytest_skip(stmt: &Stmt) -> bool {
    match stmt {
        FunctionDef(node) => node.decorator_list.iter().any(|decorator| {
            match decorator {
                // handle pytest.mark.skip decorator called with parameters
                ast::Expr::Call(call) => match call.func.as_attribute_expr() {
                    Some(attr_expr) => match attr_expr.value.as_attribute_expr() {
                        Some(nested_attr_expr) => match nested_attr_expr.value.as_name_expr() {
                            Some(name_expr) => {
                                let module = name_expr.id.as_str();
                                module == "pytest"
                                    && nested_attr_expr.attr.as_str() == "mark"
                                    && attr_expr.attr.as_str() == "skip"
                            }
                            None => false,
                        },
                        None => false,
                    },
                    None => false,
                },
                // handle pytest.mark.skip decorator called without parameters
                ast::Expr::Attribute(attr_expr) => match attr_expr.value.as_attribute_expr() {
                    Some(nested_attr_expr) => match nested_attr_expr.value.as_name_expr() {
                        Some(name_expr) => {
                            let module = name_expr.id.as_str();
                            module == "pytest"
                                && nested_attr_expr.attr.as_str() == "mark"
                                && attr_expr.attr.as_str() == "skip"
                        }
                        None => false,
                    },
                    None => false,
                },
                _ => false,
            }
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::indoc::indoc;
    use rustpython_parser::{ast, Parse};

    #[test]
    fn it_works_with_no_decorator() {
        let code = "\
        def test_sun_rises_in_the_east():
            pass
        ";
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = is_pytest_skip(ast.unwrap().first().take().unwrap());
        assert!(!result);
    }

    #[test]
    fn it_works_with_mark_without_reason() {
        let code = indoc! {"
            @pytest.mark.skip
            def test_sun_rises_in_the_west():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = is_pytest_skip(ast.unwrap().first().take().unwrap());
        assert!(result);
    }

    #[test]
    fn it_works_with_mark_called_with_reason() {
        let code = indoc! {"
            @pytest.mark.skip(reason=\"impossible test\")
            def test_sun_rises_in_the_west():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = is_pytest_skip(ast.unwrap().first().take().unwrap());
        assert!(result);
    }
}
