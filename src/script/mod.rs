pub use cpython;
use cpython::{Python, PyObject, GILGuard, PyModule};
use std::path::Path;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::hint::unreachable_unchecked;

type InterpreterResult<T> = Result<T, String>;

type ScriptID = u64;
/*
macro_rules! setup_python_hook {
    ($ss:expr, $sid:expr, $f:ident [ $( { $pname:ident : $ptype:ty = $detail:tt } )* ]) => {
        $ss.load_function($ssid, py_fn!($ss.backend.gil_guard, $f($($pname:$ptype),*)))
    };
}
*/
pub trait ScriptBackend {
    type DataType;
    type Function;
    fn load_file(&mut self, name: &str) -> InterpreterResult<ScriptID>;
    fn reload(&mut self, script: ScriptID) -> InterpreterResult<()>;
    fn clear(&mut self) -> InterpreterResult<()>;
    //fn load_function(&mut self, script: ScriptID, name: &'static str, func: Self::Function);
    fn exec(&mut self, script: ScriptID, statement: &str) -> InterpreterResult<()>;
}

pub struct PythonBackend {
    pub gil_guard: GILGuard,
    modules: HashMap<ScriptID, PyModule>,
    script_id_counter: ScriptID
}

impl PythonBackend {
    pub fn new() -> PythonBackend {
        PythonBackend {
            gil_guard: Python::acquire_gil(),
            modules: HashMap::new(),
            script_id_counter: 0
        }
    }
}

impl ScriptBackend for PythonBackend {
    type DataType = PyObject;
    type Function = PyObject;
    fn load_file(&mut self, name: &str) -> InterpreterResult<ScriptID> {
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
    fn reload(&mut self, script: ScriptID) -> InterpreterResult<()> {
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

    fn clear(&mut self) -> InterpreterResult<()> {
        self.modules.clear();
        Ok(())
    }

    fn exec(&mut self, script: ScriptID, statement: &str) -> InterpreterResult<()> {
        let python = self.gil_guard.python();
        let module = self.modules.get_mut(&script).unwrap();
        match python.run(statement, Some(&module.dict(python)), None) {
            Ok(()) => Ok(()),
            Err(_) => Err(String::from("An error occured!"))
        }
    }

}

pub struct ScriptSystem<B: ScriptBackend> {
    backend: B
}