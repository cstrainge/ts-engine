
use ts_process::{ execute::{ IsolatedProcess, ProcessError }, ipc::IpcMessage };
use crate::engine::{ CompileMode, ScriptLanguage };



#[derive(Debug)]
pub enum ScriptHostError
{
    ProcessError { error: ProcessError }
}


impl From<ProcessError> for ScriptHostError
{
    fn from(error: ProcessError) -> Self
    {
        ScriptHostError::ProcessError { error }
    }
}


pub type ScriptHostResult<T> = Result<T, ScriptHostError>;


pub struct ScriptHost<'a>
{
    parent_process: &'a mut IsolatedProcess,
    _compile_mode: CompileMode,
    _script_language: ScriptLanguage
}


impl<'a> ScriptHost<'a>
{
    /**
     * Spin up a new scripting environment for the script client, (the parent process.)
     */
    fn new(parent_process: &'a mut IsolatedProcess,
           id: i64,
           compile_mode: CompileMode,
           script_language: ScriptLanguage) -> ScriptHostResult<Self>
    {
        // TODO: Initialize the actual scripting engine.
        parent_process.respond(id, &IpcMessage::ScriptHostReady)?;

        // Create the host object instance itself.
        Ok(Self { parent_process, _compile_mode: compile_mode, _script_language: script_language })
    }

    /**
     * Runs the message loop for the script host, handling incoming IPC messages from the parent
     * process.
     */
    pub fn message_loop(&mut self) -> ScriptHostResult<()>
    {
        loop
        {
            let (id, message) = self.parent_process.receive(None)?;

            match message
            {
                IpcMessage::Ping { nonce } =>
                    {
                        self.parent_process.respond(id, &IpcMessage::Pong { nonce })?;
                    },

                IpcMessage::Shutdown =>
                    {
                        break;
                    },

                _ => {}
            }
        }

        Ok(())
    }

    /**
     * Allow the generic child process to transform itself into a script host.
     */
    pub fn execute_as_host(parent_process: &'a mut IsolatedProcess,
                           id: i64,
                           compile_mode: CompileMode,
                           script_language: ScriptLanguage) -> ScriptHostResult<()>
    {
        let mut host = ScriptHost::new(parent_process, id, compile_mode, script_language)?;

        host.message_loop()
    }
}
