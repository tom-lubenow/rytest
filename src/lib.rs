#![allow(non_local_definitions)]

use glob_match::glob_match;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

/// A Python module implemented in Rust for faster pytest collection
#[pymodule]
fn rytest_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Collector>()?;
    Ok(())
}

#[pyclass]
struct Collector {
    python_classes: Vec<String>,
    python_functions: Vec<String>,
}

#[derive(Debug, Clone)]
struct TestItem {
    name: String,
    path: String,
    line_number: usize,
    kind: TestKind,
    parameters: Option<Parameters>,
}

#[derive(Debug, Clone)]
struct Parameters {
    argnames: Vec<String>,
    argvalues: Vec<Vec<PyObject>>,
    ids: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
enum TestKind {
    Function,
    Class,
    Method,
}

impl Collector {
    fn get_config_patterns(config: &PyAny, name: &str) -> PyResult<Vec<String>> {
        let patterns = config.call_method1("getini", (name,))?;
        let patterns: Vec<String> = patterns.extract()?;
        Ok(patterns)
    }

    fn matches_pattern(name: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|pattern| {
            if pattern.contains('*') {
                glob_match(pattern, name)
            } else {
                name.starts_with(pattern)
            }
        })
    }

    /// Extract parameters from a pytest.mark.parametrize decorator
    fn extract_parameters(py: Python, line: &str) -> PyResult<Option<Parameters>> {
        // Look for @pytest.mark.parametrize or @pytest.mark.parametrize
        if !line.contains("pytest.mark.parametrize") && !line.contains("@parametrize") {
            return Ok(None);
        }

        // Extract the arguments from the decorator
        let start = line.find('(').ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(
                "Invalid parametrize decorator: missing opening parenthesis",
            )
        })?;
        let end = line.rfind(')').ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(
                "Invalid parametrize decorator: missing closing parenthesis",
            )
        })?;

        let args_str = &line[start + 1..end];

        // Parse the arguments using Python's ast module for safety
        let ast = py.import("ast")?;
        let parsed = ast.call_method1("literal_eval", (format!("({})", args_str),))?;

        // Extract argnames and argvalues
        let tuple = parsed.downcast::<PyTuple>()?;
        if tuple.len() < 2 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Invalid parametrize decorator: missing arguments",
            ));
        }

        let argnames = if tuple.get_item(0)?.is_instance_of::<PyTuple>() {
            let names = tuple.get_item(0)?.downcast::<PyTuple>()?;
            names
                .iter()
                .map(|n| n.extract())
                .collect::<PyResult<Vec<String>>>()?
        } else {
            vec![tuple.get_item(0)?.extract()?]
        };

        let argvalues = if tuple.get_item(1)?.is_instance_of::<PyList>() {
            let values = tuple.get_item(1)?.downcast::<PyList>()?;
            let mut result = Vec::new();
            for v in values.iter() {
                let values = if v.is_instance_of::<PyTuple>() {
                    let tuple = v.downcast::<PyTuple>()?;
                    tuple.iter().map(|x| x.into()).collect()
                } else {
                    vec![v.into()]
                };
                result.push(values);
            }
            result
        } else {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Invalid parametrize decorator: argvalues must be a list",
            ));
        };

        // Extract optional ids if present
        let ids = if tuple.len() > 2 {
            let kwargs = tuple.get_item(2)?.downcast::<PyDict>()?;
            if let Some(ids) = kwargs.get_item("ids")? {
                Some(ids.extract()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Some(Parameters {
            argnames,
            argvalues,
            ids,
        }))
    }

    /// Check if a line contains a test function definition and return its details
    fn parse_test_function(
        py: Python,
        lines: &[String],
        current_line: usize,
        python_functions: &[String],
    ) -> PyResult<Option<TestItem>> {
        let line = lines[current_line].trim();
        if line.starts_with("def ") {
            let name = line
                .strip_prefix("def ")
                .and_then(|s| s.split('(').next())
                .map(|s| s.trim().to_string())
                .ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err("Invalid function definition")
                })?;

            if Self::matches_pattern(&name, python_functions) {
                // Look for parametrize decorator above the function
                let mut parameters = None;
                if current_line > 0 {
                    for i in (0..current_line).rev() {
                        let prev_line = lines[i].trim();
                        if prev_line.starts_with("@") {
                            if let Some(params) = Self::extract_parameters(py, prev_line)? {
                                parameters = Some(params);
                                break;
                            }
                        } else if !prev_line.is_empty() && !prev_line.starts_with("#") {
                            break;
                        }
                    }
                }

                return Ok(Some(TestItem {
                    name,
                    path: String::new(), // Will be set by caller
                    line_number: current_line + 1,
                    kind: TestKind::Function,
                    parameters,
                }));
            }
        }
        Ok(None)
    }

    /// Check if a line contains a test class definition and return its details
    fn parse_test_class(
        line: &str,
        line_number: usize,
        python_classes: &[String],
    ) -> Option<TestItem> {
        let line = line.trim();
        if line.starts_with("class ") {
            let name = line
                .strip_prefix("class ")?
                .split(&['(', ':'])
                .next()?
                .trim()
                .to_string();

            if Self::matches_pattern(&name, python_classes) {
                return Some(TestItem {
                    name,
                    path: String::new(), // Will be set by caller
                    line_number,
                    kind: TestKind::Class,
                    parameters: None,
                });
            }
        }
        None
    }

    /// Parse a Python file and look for test functions and classes
    fn parse_file(&self, path: &str) -> PyResult<Vec<TestItem>> {
        let file = fs::File::open(path).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to read file: {}", e))
        })?;

        let reader = io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<io::Result<_>>().map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to read lines: {}", e))
        })?;

        let mut items = Vec::new();
        let mut current_class: Option<TestItem> = None;

        Python::with_gil(|py| -> PyResult<()> {
            for (line_number, line) in lines.iter().enumerate() {
                if let Some(mut func) =
                    Self::parse_test_function(py, &lines, line_number, &self.python_functions)?
                {
                    func.path = path.to_string();
                    if current_class.is_some() {
                        func.kind = TestKind::Method;
                    }
                    items.push(func);
                } else if let Some(mut class) =
                    Self::parse_test_class(line, line_number, &self.python_classes)
                {
                    class.path = path.to_string();
                    current_class = Some(class.clone());
                    items.push(class);
                }
            }
            Ok(())
        })?;

        Ok(items)
    }

    /// Create a pytest Node from a TestItem
    fn create_node(&self, py: Python, item: &TestItem, parent: &PyAny) -> PyResult<Vec<PyObject>> {
        let pytest = py.import("pytest")?;
        let mut nodes = Vec::new();

        match &item.kind {
            TestKind::Function | TestKind::Method => {
                if let Some(params) = &item.parameters {
                    // Create a parametrized node for each parameter set
                    for (i, values) in params.argvalues.iter().enumerate() {
                        let kwargs = PyDict::new(py);
                        let param_name = if let Some(ids) = &params.ids {
                            format!("{}[{}]", item.name, ids[i])
                        } else {
                            format!("{}[{}]", item.name, i)
                        };
                        kwargs.set_item("name", &param_name)?;

                        // Create a callspec with the parameter values
                        let callspec = PyDict::new(py);
                        let params_dict = PyDict::new(py);
                        for (name, value) in params.argnames.iter().zip(values.iter()) {
                            params_dict.set_item(name, value)?;
                        }
                        callspec.set_item("params", params_dict)?;
                        kwargs.set_item("callspec", callspec)?;

                        let func = pytest.getattr("Function")?;
                        let node = func
                            .call_method("from_parent", (parent,), Some(&kwargs))?
                            .into_py(py);

                        // Set additional attributes
                        let node_ref = node.as_ref(py);
                        node_ref.setattr("_nodeid", format!("{}::{}", &item.path, &param_name))?;
                        node_ref
                            .setattr("_location", (&item.path, item.line_number, &param_name))?;

                        nodes.push(node);
                    }
                } else {
                    // Create a single node for non-parametrized function
                    let kwargs = PyDict::new(py);
                    kwargs.set_item("name", &item.name)?;
                    let func = pytest.getattr("Function")?;
                    let node = func
                        .call_method("from_parent", (parent,), Some(&kwargs))?
                        .into_py(py);

                    // Set additional attributes
                    let node_ref = node.as_ref(py);
                    node_ref.setattr("_nodeid", format!("{}::{}", &item.path, &item.name))?;
                    node_ref.setattr("_location", (&item.path, item.line_number, &item.name))?;

                    nodes.push(node);
                }
            }
            TestKind::Class => {
                let kwargs = PyDict::new(py);
                kwargs.set_item("name", &item.name)?;
                let class = pytest.getattr("Class")?;
                let node = class
                    .call_method("from_parent", (parent,), Some(&kwargs))?
                    .into_py(py);

                // Set additional attributes
                let node_ref = node.as_ref(py);
                node_ref.setattr("_nodeid", format!("{}::{}", &item.path, &item.name))?;
                node_ref.setattr("_location", (&item.path, item.line_number, &item.name))?;

                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    /// Check if a directory is a Python package (has __init__.py)
    fn is_package_dir(path: &Path) -> bool {
        path.join("__init__.py").is_file()
    }

    /// Create a Package node for a directory
    fn create_package_node(&self, py: Python, path: &Path, parent: &PyAny) -> PyResult<PyObject> {
        let pytest = py.import("pytest")?;
        let pathlib = py.import("pathlib")?;
        let path_obj = if parent.hasattr("path")? {
            // If parent has a path attribute, use it to create a relative path
            let parent_path = parent.getattr("path")?;
            parent_path.call_method1(
                "__truediv__",
                (path.file_name().unwrap().to_str().unwrap(),),
            )?
        } else {
            // Otherwise create a new Path object
            pathlib.call_method1("Path", (path.to_str().unwrap(),))?
        };
        let kwargs = PyDict::new(py);
        kwargs.set_item("path", path_obj)?;
        let package = pytest.getattr("Package")?;
        Ok(package
            .call_method("from_parent", (parent,), Some(&kwargs))?
            .into())
    }

    /// Create a Module node for a file
    fn create_module_node(&self, py: Python, path: &Path, parent: &PyAny) -> PyResult<PyObject> {
        let pytest = py.import("pytest")?;
        let pathlib = py.import("pathlib")?;
        let path_obj = if parent.hasattr("path")? {
            // If parent has a path attribute, use it to create a relative path
            let parent_path = parent.getattr("path")?;
            parent_path.call_method1(
                "__truediv__",
                (path.file_name().unwrap().to_str().unwrap(),),
            )?
        } else {
            // Otherwise create a new Path object
            pathlib.call_method1("Path", (path.to_str().unwrap(),))?
        };
        let kwargs = PyDict::new(py);
        kwargs.set_item("path", path_obj)?;
        let module = pytest.getattr("Module")?;
        Ok(module
            .call_method("from_parent", (parent,), Some(&kwargs))?
            .into())
    }

    /// Collect a directory recursively
    fn collect_dir(&self, py: Python, path: &Path, parent: &PyAny) -> PyResult<Option<PyObject>> {
        // Check for infinite recursion by looking at parent chain
        let mut current = parent;
        while let Ok(parent_path) = current.getattr("path") {
            if let Ok(parent_path_str) = parent_path.extract::<String>() {
                if parent_path_str == path.to_str().unwrap_or("") {
                    // Found same path in parent chain, stop recursion
                    return Ok(None);
                }
            }
            if let Ok(parent_obj) = current.getattr("parent") {
                if parent_obj.is_none() {
                    break;
                }
                current = parent_obj;
            } else {
                break;
            }
        }

        // Check if it's a package directory
        if Self::is_package_dir(path) {
            // Create a Package node
            let package = self.create_package_node(py, path, parent)?;

            // Collect __init__.py first
            let init_path = path.join("__init__.py");
            if let Some(items) = self.parse_file(init_path.to_str().unwrap()).ok() {
                for item in items {
                    let _ = self.create_node(py, &item, package.as_ref(py))?;
                }
            }

            // Then collect other Python files
            for entry in fs::read_dir(path).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!("Failed to read directory: {}", e))
            })? {
                let entry = entry.map_err(|e| {
                    pyo3::exceptions::PyIOError::new_err(format!(
                        "Failed to read directory entry: {}",
                        e
                    ))
                })?;
                let entry_path = entry.path();

                if entry_path.is_file() {
                    if entry_path.file_name().unwrap() != "__init__.py" {
                        let _ = self.pytest_collect_file(
                            entry_path.to_str().unwrap(),
                            package.clone_ref(py),
                        )?;
                    }
                } else if entry_path.is_dir() {
                    let _ = self.collect_dir(py, &entry_path, package.as_ref(py))?;
                }
            }

            Ok(Some(package))
        } else {
            // Not a package directory, just collect Python files
            let mut collected = false;
            for entry in fs::read_dir(path).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!("Failed to read directory: {}", e))
            })? {
                let entry = entry.map_err(|e| {
                    pyo3::exceptions::PyIOError::new_err(format!(
                        "Failed to read directory entry: {}",
                        e
                    ))
                })?;
                let entry_path = entry.path();

                if entry_path.is_file() {
                    if let Some(_) =
                        self.pytest_collect_file(entry_path.to_str().unwrap(), parent.into())?
                    {
                        collected = true;
                    }
                }
            }

            Ok(if collected { Some(parent.into()) } else { None })
        }
    }
}

#[pymethods]
impl Collector {
    #[new]
    fn new(config: PyObject) -> PyResult<Self> {
        Python::with_gil(|py| {
            let config = config.as_ref(py);

            let python_classes = Self::get_config_patterns(config, "python_functions")?;
            let python_functions = Self::get_config_patterns(config, "python_functions")?;

            Ok(Collector {
                python_classes,
                python_functions,
            })
        })
    }

    /// Check if a file should be collected for tests
    fn pytest_collect_file(&self, path: &str, parent: PyObject) -> PyResult<Option<PyObject>> {
        let path = Path::new(path);

        // Only process .py files
        if path.extension().and_then(|s| s.to_str()) != Some("py") {
            return Ok(None);
        }

        Python::with_gil(|py| {
            // Parse the file to find test items
            let items = self.parse_file(path.to_str().unwrap())?;

            // Create a Module node
            let module = self.create_module_node(py, path, parent.as_ref(py))?;

            // Create child nodes for each test item
            for item in items {
                let nodes = self.create_node(py, &item, module.as_ref(py))?;
                for node in nodes {
                    // Set the module attribute on the node
                    let node_ref = node.as_ref(py);
                    if let Ok(module_attr) = node_ref.getattr("module") {
                        if module_attr.is_none() {
                            node_ref.setattr("module", module.as_ref(py))?;
                        }
                    }
                    // Set the parent attribute
                    if let Ok(parent_attr) = node_ref.getattr("parent") {
                        if parent_attr.is_none() {
                            node_ref.setattr("parent", module.as_ref(py))?;
                        }
                    }
                }
            }

            // Set the parent attribute on the module
            let module_ref = module.as_ref(py);
            if let Ok(parent_attr) = module_ref.getattr("parent") {
                if parent_attr.is_none() {
                    module_ref.setattr("parent", parent.as_ref(py))?;
                }
            }

            Ok(Some(module))
        })
    }

    /// Check if a directory should be collected for tests
    fn pytest_collect_directory(&self, path: &str, parent: PyObject) -> PyResult<Option<PyObject>> {
        Python::with_gil(|py| self.collect_dir(py, Path::new(path), parent.as_ref(py)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        Python::with_gil(|py| {
            let m = pyo3::wrap_pymodule!(rytest_core);
            let _module = m(py);
        });
    }

    #[test]
    fn test_pattern_matching() {
        let patterns = vec!["test_*.py".to_string(), "*_test.py".to_string()];
        assert!(Collector::matches_pattern("test_example.py", &patterns));
        assert!(Collector::matches_pattern("example_test.py", &patterns));
        assert!(!Collector::matches_pattern("example.py", &patterns));

        let class_patterns = vec!["Test".to_string()];
        assert!(Collector::matches_pattern("TestExample", &class_patterns));
        assert!(!Collector::matches_pattern("Example", &class_patterns));
    }

    #[test]
    fn test_parametrize_parsing() {
        Python::with_gil(|py| {
            // Simple parametrize
            let line = r#"@pytest.mark.parametrize("value", [1, 2, 3])"#;
            let params = Collector::extract_parameters(py, line).unwrap().unwrap();
            assert_eq!(params.argnames, vec!["value"]);
            assert_eq!(params.argvalues.len(), 3);
            assert!(params.ids.is_none());

            // Multiple parameters
            let line = r#"@pytest.mark.parametrize(("x", "y"), [(1, 2), (3, 4)])"#;
            let params = Collector::extract_parameters(py, line).unwrap().unwrap();
            assert_eq!(params.argnames, vec!["x", "y"]);
            assert_eq!(params.argvalues.len(), 2);
            assert!(params.ids.is_none());

            // With ids
            let line = r#"@pytest.mark.parametrize("value", [1, 2, 3], ids=["a", "b", "c"])"#;
            let params = Collector::extract_parameters(py, line).unwrap().unwrap();
            assert_eq!(params.argnames, vec!["value"]);
            assert_eq!(params.argvalues.len(), 3);
            assert_eq!(params.ids.unwrap(), vec!["a", "b", "c"]);
        });
    }

    #[test]
    fn test_file_collection() {
        Python::with_gil(|py| {
            // Create a mock config
            let config: Py<PyAny> = PyDict::new(py).into_py(py);
            let collector = Collector {
                python_classes: vec!["Test".to_string()],
                python_functions: vec!["test_".to_string()],
            };
            let parent = py.None();

            // Create a temporary test file
            let test_content = r#"
@pytest.mark.parametrize("value", [1, 2, 3])
def test_parametrized(value):
    assert value > 0

def test_simple():
    assert True

class TestExample:
    def test_method(self):
        pass
"#;
            let temp_dir = std::env::temp_dir();
            let test_file = temp_dir.join("test_temp.py");
            fs::write(&test_file, test_content).unwrap();

            // Should collect our test file
            let result = collector
                .pytest_collect_file(test_file.to_str().unwrap(), parent)
                .unwrap();
            assert!(result.is_some());

            // Clean up
            fs::remove_file(test_file).unwrap();
        });
    }

    #[test]
    fn test_package_collection() {
        Python::with_gil(|py| {
            // Create a mock config
            let config: Py<PyAny> = PyDict::new(py).into_py(py);
            let collector = Collector {
                python_classes: vec!["Test".to_string()],
                python_functions: vec!["test_".to_string()],
            };
            let parent = py.None();

            // Create a temporary package directory
            let temp_dir = std::env::temp_dir();
            let pkg_dir = temp_dir.join("test_pkg");
            fs::create_dir(&pkg_dir).unwrap();

            // Create __init__.py
            let init_content = r#"
def test_init():
    assert True
"#;
            fs::write(pkg_dir.join("__init__.py"), init_content).unwrap();

            // Create a test module
            let test_content = r#"
@pytest.mark.parametrize("value", [1, 2, 3])
def test_parametrized(value):
    assert value > 0

def test_example():
    assert True

class TestExample:
    def test_method(self):
        pass
"#;
            fs::write(pkg_dir.join("test_module.py"), test_content).unwrap();

            // Should collect our package
            let result = collector
                .pytest_collect_directory(pkg_dir.to_str().unwrap(), parent)
                .unwrap();
            assert!(result.is_some());

            // Clean up
            fs::remove_dir_all(pkg_dir).unwrap();
        });
    }
}
