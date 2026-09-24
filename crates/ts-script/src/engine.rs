
use ts_process::{ execute::{ IsolatedProcess, ProcessError },
                  ipc::IpcMessage };



pub type CompileMode = ts_process::ipc::CompileMode;

pub type ScriptLanguage = ts_process::ipc::ScriptLanguage;


pub struct ScriptEngine
{
    child_process: IsolatedProcess
}


#[derive(Debug)]
pub enum ScriptEngineError
{
    ProcessError { error: ProcessError },
    EngineFailedStart,
    EngineNotAlive
}


impl From<ProcessError> for ScriptEngineError
{
    fn from(error: ProcessError) -> Self
    {
        ScriptEngineError::ProcessError { error }
    }
}


impl ScriptEngine
{
    pub fn new(compile_mode: CompileMode, script_language: ScriptLanguage) -> Result<Self, ScriptEngineError>
    {
        let mut child_process = IsolatedProcess::new()?;

        let result = child_process.send_and_receive(&IpcMessage::InitAsScriptHost
            {
                compile_mode,
                script_language
            },
            None)?;

        if result != IpcMessage::ScriptHostReady
        {
            return Err(ScriptEngineError::EngineFailedStart);
        }

        Ok(Self
        {
            child_process
        })
    }

    pub fn keep_alive(&mut self) -> Result<(), ScriptEngineError>
    {
        let alive = self.child_process.is_alive(None)?;

        if !alive
        {
            Err(ScriptEngineError::EngineNotAlive)
        }
        else
        {
            Ok(())
        }
    }
}


impl Drop for ScriptEngine
{
    fn drop(&mut self)
    {
        let _ = self.child_process.shutdown();

        println!("Script engine has been shut down.");
    }
}
