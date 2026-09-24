
use std::{ collections::VecDeque,
           fs::File,
           io::{ self, Read, Write },
           os::fd::{ AsFd, AsRawFd },
           process::{ Child, Command, Stdio },
           sync::{ atomic::{ AtomicI64, Ordering }, Mutex },
           time::{ Duration, Instant },
           thread::sleep };

use crate::ipc::{ IpcMessage, IpcError, IpcPacket };



const MAX_IPC_PACKET_SIZE: usize = 64 * 1024 * 1024;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);


#[derive(Debug)]
pub enum ProcessError
{
    StartFailed { message: String },
    ProcessDead,
    IpcError { error: IpcError },
    IoError { error: io::Error },
    ConnectionTimeout,
    UnexpectedResponse
}


impl From<IpcError> for ProcessError
{
    fn from(error: IpcError) -> Self
    {
        ProcessError::IpcError { error }
    }
}


impl From<io::Error> for ProcessError
{
    fn from(error: io::Error) -> Self
    {
        ProcessError::IoError { error }
    }
}


pub type ProcessResult<T> = Result<T, ProcessError>;


trait IpcReader: Read + Send + AsRawFd
{
}


impl<T> IpcReader for T
    where T: Read + Send + AsRawFd
{
}


trait IpcWriter: Write + Send + AsRawFd
{
}


impl<T> IpcWriter for T
    where T: Write + Send + AsRawFd
{
}



pub struct IsolatedProcess
{
    is_parent: bool,
    next_message_id: AtomicI64,
    message_queue: VecDeque<IpcPacket>,
    reader: Mutex<Box<dyn IpcReader>>,
    writer: Mutex<Box<dyn IpcWriter>>,
    child: Option<Child>
}


impl IsolatedProcess
{
    /**
     * Create and run a new isolated process. Called from the parent process.
     */
    pub fn new() -> ProcessResult<Self>
    {
        let executable = std::env::current_exe()
            .map_err(|error|
                {
                    ProcessError::StartFailed { message: error.to_string() }
                })?;

        let mut child = Command::new(executable)
            .arg("--ts-process-child")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|error|
                {
                    ProcessError::StartFailed { message: error.to_string() }
                })?;

        let writer = child.stdin
                          .take()
                          .ok_or_else(||
            ProcessError::StartFailed { message: "Failed to take child stdin".to_string() })?;

        let reader = child.stdout
                          .take()
                          .ok_or_else(||
            ProcessError::StartFailed { message: "Failed to take child stdout".to_string() })?;

        Ok(IsolatedProcess
            {
                is_parent: true,
                next_message_id: AtomicI64::new(1),
                message_queue: VecDeque::new(),
                reader: Mutex::new(Box::new(reader)),
                writer: Mutex::new(Box::new(writer)),
                child: Some(child)
            })
    }

    /**
     * Construct a reference to the parent process from the child process.
     */
    pub fn new_from_child() -> ProcessResult<Self>
    {
        let stdin = io::stdin();
        let stdout = io::stdout();

        let reader = File::from(stdin.as_fd().try_clone_to_owned()?);
        let writer = File::from(stdout.as_fd().try_clone_to_owned()?);

        Ok(IsolatedProcess
            {
                is_parent: false,
                next_message_id: AtomicI64::new(-1),
                message_queue: VecDeque::new(),
                reader: Mutex::new(Box::new(reader)),
                writer: Mutex::new(Box::new(writer)),
                child: None
            })
    }

    /**
     * Check if the isolated process is still alive and responding to messages.
     */
    pub fn is_alive(&mut self, timeout: Option<Duration>) -> ProcessResult<bool>
    {
        if let Some(child) = self.child.as_mut()
        {
            if child.try_wait()?.is_some()
            {
                return Ok(false);
            }
        }

        // Create a new unique ping message with a random nonce.
        let (ping, nonce) = IpcMessage::new_ping();

        // Send the ping message to the isolated process and check for a pong response.
        let response = self.send_and_receive(&ping, timeout)?;

        match response
        {
            IpcMessage::Pong { nonce: response_nonce } if response_nonce == nonce => Ok(true),
            _ => Err(ProcessError::UnexpectedResponse)
        }
    }

    /**
     * Attempt request a shutdown. The semantics of this differ depending on whether we're the
     * parent or child process.
     *
     * From parent to child, this will request the child to shut down.
     * From child to parent it is a notification that the child intends to shut down.
     */
    pub fn shutdown(&mut self,
                    message_timeout: Option<Duration>,
                    wait_timeout: Option<Duration>) -> ProcessResult<()>
    {
        // Implementation for killing the isolated process would go here.
        if self.is_alive(message_timeout)?
        {
            let _ = self.send(&IpcMessage::Shutdown)?;
        }

        // Wait a short duration to allow the other process to handle the shutdown message.
        let timeout = wait_timeout.unwrap_or(Duration::from_millis(500));
        sleep(timeout);

        // Check if the process is still alive after the shutdown message.
        let result = self.is_alive(None);

        if let Ok(true) = result
        {
            // The process is still alive after the shutdown message. Send a sigkill to forcefully
            // terminate it.
            if let Some(child) = self.child.as_mut()
            {
                let _ = child.kill();
            }
        }

        Ok(())
    }

    /**
     * Attempt to send an IpcMessage to the isolated process.
     */
    pub fn send(&self, message: &IpcMessage) -> ProcessResult<i64>
    {
        let message_id = self.get_next_id();
        self.transmit_packet(IpcPacket::new(message_id, message))?;

        Ok(message_id)
    }

    /**
     * Respond to a message from the other side of the connection.
     */
    pub fn respond(&self, original_id: i64, message: &IpcMessage) -> ProcessResult<()>
    {
        self.transmit_packet(IpcPacket::new(original_id, message))?;

        Ok(())
    }

    /**
     * Attempt to transmit a packet to the other process.
     */
    fn transmit_packet(&self, packet: IpcPacket) -> ProcessResult<()>
    {
        let payload = packet.to_wire();
        let size = payload.len().to_le_bytes();

        let mut bytes = Vec::with_capacity(size.len() + payload.len());
        bytes.extend_from_slice(&size);
        bytes.extend_from_slice(&payload);

        self.transmit_bytes(bytes)
    }

    /**
     * Transmit raw bytes to the other process.
     */
    fn transmit_bytes(&self, bytes: Vec<u8>) -> ProcessResult<()>
    {
        let mut writer = self.writer
                             .lock()
                             .expect("Failed to lock writer");

        writer.write_all(&bytes)?;
        writer.flush()?;

        Ok(())
    }

    /**
     * Send a message to the isolated process and wait for a response within the specified
     * timeout.
     */
    pub fn send_and_receive(&mut self,
                            message: &IpcMessage,
                            timeout: Option<Duration>) -> ProcessResult<IpcMessage>
    {
        // Compute the deadline for the response based on the current time and the specified
        // timeout.
        let deadline = Instant::now() + timeout.unwrap_or(DEFAULT_TIMEOUT);

        // Get send the message and get it's message ID for tracking the response.
        let send_id = self.send(message)?;

        // Keep trying to receive the response until we either get it or hit the timeout.
        loop
        {
            // Check if the response for the sent message is already in the queue between the last
            // send and now. If it is, we can return it immediately.
            if let Some(index) = self.message_queue
                                     .iter()
                                     .position(|packet| packet.id == send_id)
            {
                let packet = self.message_queue.remove(index).unwrap();
                return Ok(packet.message);
            }

            // The message wasn't in the queue so try to pull it directly from the other process.
            let now = Instant::now();

            if now >= deadline
            {
                return Err(ProcessError::ConnectionTimeout);
            }

            let remaining_time = deadline - now;
            let (receive_id, response) = self.receive_new(Some(remaining_time))?;

            // Make sure the message received is the one we are waiting for. If not, queue it for
            // later.
            if send_id != receive_id
            {
                self.message_queue.push_back(IpcPacket::new(receive_id, &response));
            }
            else
            {
                return Ok(response);
            }
        }
    }

    /**
     * Check to see if there are any pending messages from the other process.
     */
    pub fn check_available(&self) -> ProcessResult<bool>
    {
        // Implementation for checking if IsolatedProcess is available would go here.
        Ok(true)
    }


    pub fn receive(&mut self, timeout: Option<Duration>) -> ProcessResult<(i64, IpcMessage)>
    {
        if !self.message_queue.is_empty()
        {
            let packet = self.message_queue.pop_front().expect("Message queue should not be empty");
            Ok((packet.id, packet.message))
        }
        else
        {
            self.receive_new(timeout)
        }
    }

    /**
     * Attempt to receive a message from the isolated process within the specified timeout.
     */
    fn receive_new(&self, timeout: Option<Duration>) -> ProcessResult<(i64, IpcMessage)>
    {
        let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
        let deadline = Instant::now() + timeout;

        let mut reader = self.reader.lock().expect("Failed to lock reader");
        let mut size_bytes = [0u8; size_of::<usize>()];

        Self::read_exact_until(&mut **reader, &mut size_bytes, deadline)?;

        let packet_size = usize::from_le_bytes(size_bytes);

        if packet_size > MAX_IPC_PACKET_SIZE
        {
            return Err(ProcessError::IpcError
                {
                    error: IpcError::DecodeError
                        {
                            message: format!("IPC packet size {} exceeds maximum size {}.",
                                             packet_size,
                                             MAX_IPC_PACKET_SIZE)
                        }
                });
        }

        let mut packet_bytes = vec![0u8; packet_size];

        Self::read_exact_until(&mut **reader, &mut packet_bytes, deadline)?;

        let packet = IpcPacket::from_wire(&packet_bytes)?;

        Ok((packet.id, packet.message))
    }

    /**
     * Read exactly the number of bytes required into the buffer, waiting until the deadline.
     */
    fn read_exact_until(reader: &mut dyn IpcReader,
                        buffer: &mut [u8],
                        deadline: Instant) -> ProcessResult<()>
    {
        let mut offset = 0;

        while offset < buffer.len()
        {
            let remaining = deadline.saturating_duration_since(Instant::now());

            if remaining.is_zero()
            {
                return Err(ProcessError::ConnectionTimeout);
            }

            let mut fd = libc::pollfd
                {
                    fd: reader.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0
                };

            let timeout = remaining.as_millis()
                                   .clamp(1, i32::MAX as u128) as i32;

            let result = unsafe { libc::poll(&mut fd, 1, timeout) };

            match result
            {
                0 => return Err(ProcessError::ConnectionTimeout),

                result if result < 0 =>
                    {
                        let error = io::Error::last_os_error();

                        if error.kind() == io::ErrorKind::Interrupted
                        {
                            continue;
                        }

                        return Err(error.into());
                    },

                _ => {}
            }

            match reader.read(&mut buffer[offset..])
            {
                Ok(0) => return Err(ProcessError::ProcessDead),
                Ok(size) => offset += size,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error.into())
            }
        }

        Ok(())
    }

    /**
     * Get the next unique transmission message ID.
     */
    fn get_next_id(&self) -> i64
    {
        self.next_message_id
            .try_update(
                Ordering::Relaxed,
                Ordering::Relaxed,
                |id|
                {
                    Some(if self.is_parent
                    {
                        let next_id = id.wrapping_add(1);

                        if next_id <= 0 { 1 } else { next_id }
                    }
                    else
                    {
                        let next_id = id.wrapping_sub(1);

                        if next_id >= 0 { -1 } else { next_id }
                    })
                })
            .expect("Message ID update cannot fail.")
    }
}
