pub use cpython;
use cpython::{Python, PyObject, GILGuard, PyModule, FromPyObject};
use std::path::Path;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::hint::unreachable_unchecked;

pub type InterpreterResult<T> = Result<T, String>;

pub type ScriptID = u64;
/*
macro_rules! setup_python_hook {
    ($ss:expr, $sid:expr, $f:ident [ $( { $pname:ident : $ptype:ty = $detail:tt } )* ]) => {
        $ss.load_function($ssid, py_fn!($ss.backend.gil_guard, $f($($pname:$ptype),*)))
    };
}
*/

pub struct PythonInterpreter {
    pub gil_guard: GILGuard,
    modules: HashMap<ScriptID, PyModule>,
    script_id_counter: ScriptID
}

impl PythonInterpreter {
    pub fn new() -> PythonInterpreter {
        PythonInterpreter {
            gil_guard: Python::acquire_gil(),
            modules: HashMap::new(),
            script_id_counter: 0
        }
    }

    /// This is a helper function that appends a path to `sys.path` to allow imports from other locations.
    /// This doesn't happen by default, for safety reasons.
    pub fn include(&mut self, path: &'static str) -> InterpreterResult<()> {
        let python = self.gil_guard.python();
        //Note: Potential injection vulnerability. With a &'static str it shouldn't be a problem though.
        let command = format!("import sys\nsys.path.append(\"{}\")", path);
        match python.run(command.as_str(), None, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error adding include path: {:?}", e))
        }
    }

    pub fn load_module(&mut self, name: &str) -> InterpreterResult<ScriptID> {
        let python = self.gil_guard.python();
        match python.import(name) {
            Ok(module) => {
                self.modules.insert(self.script_id_counter, module);
                self.script_id_counter += 1;
                Ok(self.script_id_counter - 1) // It's -1 because we just incremented it
            },
            Err(error) => {
                Err(format!("Error loading Python script: {:?}", error))
            }
        }
    }
    pub fn reload(&mut self, script: ScriptID) -> InterpreterResult<()> {
        let python = self.gil_guard.python();
        match self.modules.get(&script) {
            Some(m) => {
                let name = m.name(python).unwrap();
                match python.import(name) {
                    Ok(module) => {
                        self.modules.insert(script, module);
                        Ok(())
                    },
                    Err(error) => {
                        Err(format!("Error re-loading Python script: {:?}", error))
                    }
                }
            },
            None => {
                Err(format!("Attempted to reload non-existent script ID ({})", script))
            }
        }
    }

    pub fn get_value(&mut self, script: ScriptID, variable_name: &str) -> InterpreterResult<PyObject> {
        let python = self.gil_guard.python();
        match self.modules.get(&script) {
            Some(m) => {
                match m.dict(python).get_item(python, variable_name) {
                    Some(v) => Ok(v),
                    None => Err(format!("Script with id ({}) contains no variable called '{}'", script, variable_name))
                }
            },
            None => {
                Err(format!("Atempted to use non-existent script ID ({})", script))
            }
        }
    }

    pub fn convert<'a, V>(&mut self, py_obj: &'a PyObject) -> InterpreterResult<Box<V>>
    where V: FromPyObject<'a> {
        let python = self.gil_guard.python();
        match py_obj.extract::<V>(python) {
            Ok(v) => Ok(Box::new(v)),
            Err(e) => Err(format!("Could not convert object to desired type: {:?}", e))
        }
    }

    pub fn clear(&mut self) -> InterpreterResult<()> {
        self.modules.clear();
        Ok(())
    }

    pub fn exec(&mut self, script: ScriptID, statement: &str) -> InterpreterResult<()> {
        let python = self.gil_guard.python();
        let module = self.modules.get_mut(&script).unwrap();
        match python.run(statement, Some(&module.dict(python)), None) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Error when running '{}': {:?}", statement, e))
        }
    }

}

/// A subsystem that sends ECS data to Python scripts.
pub struct PythonScriptSystem {
    backend: PythonInterpreter
}

/// This is currently unimplemented
pub struct LuaScriptSystem {}