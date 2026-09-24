
use std::thread::sleep;

use ts_process::{ ipc::IpcMessage, execute::IsolatedProcess };



fn run_as_parent()
{
    let mut child = IsolatedProcess::new().unwrap();
    let message = child.send_and_receive(&IpcMessage::Ping { value: 1024 }, None).unwrap();

    println!("Received message from child: {:?}", message);

    child.send(&IpcMessage::Shutdown).unwrap();

    sleep(std::time::Duration::from_millis(100));

    if child.is_alive(None).unwrap()
    {
        println!("Child is still alive.");
    }
    else
    {
        println!("Child has terminated.");
    }
}


fn run_as_child()
{
    let mut parent = IsolatedProcess::new_from_child().unwrap();

    loop
    {
        let (id, message) = parent.receive(None).unwrap();

        match message
        {
            IpcMessage::Ping { value } =>
                {
                    parent.respond(id, &IpcMessage::Pong { value }).unwrap();
                },

            IpcMessage::Shutdown =>
                {
                    break;
                },

            _ => {}
        }
    }
}


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
