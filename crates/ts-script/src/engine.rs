
use ts_process::{ capabilities::Capabilities,
                  execute::{ IsolatedProcess, ProcessError },
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
    CapabilitiesApplyFailed,
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
        // Create the child process that will be the script engine itself.
        let mut child_process = IsolatedProcess::new()?;

        // Apply default capabilities to the child process.
        let capabilities = Capabilities::default();

        let result = child_process.send_and_receive(&IpcMessage::ApplyCapabilities
                {
                    capabilities
                },
                None)?;

        if result != IpcMessage::CapabilitiesApplied
        {
            return Err(ScriptEngineError::CapabilitiesApplyFailed);
        }

        // Now tell the child process it is being initialized as a script host.
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

        // At this point, the script engine has been successfully initialized and is ready to use.
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
        let _ = self.child_process.shutdown(None, None);
    }
}
