
use std::thread::sleep;

use ts_script::{ engine::{ CompileMode, ScriptEngine, ScriptLanguage }, host::ScriptHost };
use ts_process::{ capabilities::apply_capabilities, ipc::IpcMessage, execute::IsolatedProcess };



/**
 * Runs as the child process, handling incoming IPC messages from the parent process. It's the
 * parent process that tells the child process what role it should assume.
 */
fn run_as_child()
{
    // Construct a link to the parent process.
    let mut parent = IsolatedProcess::new_from_child().unwrap();
    let mut capabilities_applied = false;

    loop
    {
        // Wait for the next message from the parent process, if we receive an error, break the
        // loop and allow ourself to exit gracefully.
        let result = parent.receive(None);

        if result.is_err()
        {
            break;
        }

        // Unwrap the result since we already checked for errors above. We need to know what message
        // was actually sent.
        let (id, message) = result.unwrap();

        match message
        {
            // Apply the capability set as sent by the parent process.
            IpcMessage::ApplyCapabilities { capabilities } =>
                {
                    if capabilities_applied
                    {
                        eprintln!("Capabilities have already been applied, aborting.");
                        break;
                    }

                    capabilities_applied = true;
                    apply_capabilities(&capabilities);
                    parent.respond(id, &IpcMessage::CapabilitiesApplied).unwrap();

                },

            // We're being told we're supposed to initialize as a script host. Do so now. When this
            // returns, it's because we've been told to shutdown.
            IpcMessage::InitAsScriptHost { compile_mode, script_language } =>
                {
                    if !capabilities_applied
                    {
                        eprintln!("Capabilities have not been applied, aborting.");
                        break;
                    }

                    ScriptHost::execute_as_host(&mut parent,
                                                id,
                                                compile_mode,
                                                script_language).unwrap();
                    break;
                },

            // Handle a standard ping message from the parent process. We're still alive but we
            // don't know our specialization yet.
            IpcMessage::Ping { nonce } =>
                {
                    parent.respond(id, &IpcMessage::Pong { nonce }).unwrap();
                },

            // We're still unspecialized but we've received a shutdown message, so we should exit.
            IpcMessage::Shutdown =>
                {
                    break;
                },

            // Ignore all other messages until we are specialized.
            _ => {}
        }
    }
}


/**
 * Runs as the parent process. Scripts run within child processes that are instances of this engine.
 */
fn run_as_parent()
{
    let mut engine = ScriptEngine::new(CompileMode::Debug, ScriptLanguage::TypeScript).unwrap();

    println!("Script engine initialized successfully.");

    sleep(std::time::Duration::from_millis(100));

    engine.keep_alive().unwrap();

    println!("Script engine is alive.");
}


/**
 * Start up and determine whether to run as a child or parent process.
 */
fn main()
{
    if std::env::args().any(|arg| arg == "--ts-process-child")
    {
        run_as_child();
    }
    else
    {
        run_as_parent();
    }
}
