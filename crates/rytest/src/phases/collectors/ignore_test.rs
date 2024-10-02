use rustpython_parser::ast;
use rustpython_parser::ast::Stmt;
use rustpython_parser::ast::Stmt::FunctionDef;

pub fn is_pytest_fixture(stmt: &Stmt) -> bool {
    match stmt {
        FunctionDef(node) => node.decorator_list.iter().any(|decorator| {
            match decorator {
                // handle pytest.fixture decorator called with parameters
                ast::Expr::Call(call) => match call.func.as_attribute_expr() {
                    Some(attr_expr) => match attr_expr.value.as_name_expr() {
                        Some(name_expr) => {
                            let module = name_expr.id.as_str();
                            module == "pytest" && attr_expr.attr.as_str() == "fixture"
                        }
                        None => false,
                    },
                    None => false,
                },
                // handle pytest.fixture decorator called without parameters
                ast::Expr::Attribute(attr_expr) => match attr_expr.value.as_name_expr() {
                    Some(name_expr) => {
                        let module = name_expr.id.as_str();
                        module == "pytest" && attr_expr.attr.as_str() == "fixture"
                    }
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
    fn it_works_with_non_fixtures() {
        let code = "\
        def test_not_a_fixture():
            pass
        ";
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = is_pytest_fixture(ast.unwrap().first().take().unwrap());
        assert!(!result);
    }

    #[test]
    fn it_works_with_fixtures_with_pytest_decorator() {
        let code = indoc! {"
            @pytest.fixture
            def test_fixture():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = is_pytest_fixture(ast.unwrap().first().take().unwrap());
        assert!(result);
    }

    #[test]
    fn it_works_with_fixtures_with_pytest_decorator_as_called() {
        let code = indoc! {"
            @pytest.fixture(name=\"testing\")
            def test_fixture():
                pass
        "};
        let ast = ast::Suite::parse(code, "<embedded>");
        let result = is_pytest_fixture(ast.unwrap().first().take().unwrap());
        assert!(result);
    }
}
