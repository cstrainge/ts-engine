
use std::{ fmt::{ self, Display, Formatter }, random::random };



#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CompileMode
{
    Debug,
    Release
}


impl TryFrom<u8> for CompileMode
{
    type Error = IpcError;

    fn try_from(value: u8) -> Result<Self, Self::Error>
    {
        match value
        {
            0 => Ok(CompileMode::Debug),
            1 => Ok(CompileMode::Release),

            _ => Err(IpcError::DecodeError
                {
                    message: format!("Unknown compile mode: {}.", value)
                })
        }
    }
}



#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScriptLanguage
{
    JavaScript,
    TypeScript
}



impl TryFrom<u8> for ScriptLanguage
{
    type Error = IpcError;

    fn try_from(value: u8) -> Result<Self, Self::Error>
    {
        match value
        {
            0 => Ok(ScriptLanguage::JavaScript),
            1 => Ok(ScriptLanguage::TypeScript),

            _ => Err(IpcError::DecodeError
                {
                    message: format!("Unknown script language: {}.", value)
                })
        }
    }
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcMessage
{
    InitAsScriptHost { compile_mode: CompileMode, script_language: ScriptLanguage },
    Shutdown,
    Ping { nonce: u32 },
    Pong { nonce: u32 }
}


impl IpcMessage
{
    pub fn new_ping() -> (IpcMessage, u32)
    {
        // Generate a random nonce for the ping message.
        let nonce = random::<u32>(..);

        (IpcMessage::Ping { nonce }, nonce)
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcError
{
    DecodeError{ message: String }
}


impl Display for IpcError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        match self
        {
            IpcError::DecodeError{ message } => write!(f, "IPC decode error: {}", message)
        }
    }
}


impl IpcMessage
{
    fn id(&self) -> u8
    {
        match self
        {
            IpcMessage::InitAsScriptHost { .. } => 0,
            IpcMessage::Shutdown => 1,
            IpcMessage::Ping { .. } => 2,
            IpcMessage::Pong { .. } => 3,
        }
    }

    pub fn to_wire(&self) -> Vec<u8>
    {
        match self
        {
            IpcMessage::InitAsScriptHost { compile_mode, script_language } =>
                {
                    let mut data = vec![0; 3];

                    data[0] = self.id();
                    data[1] = *compile_mode as u8;
                    data[2] = *script_language as u8;

                    data
                },

            IpcMessage::Shutdown => vec![self.id()],

              IpcMessage::Ping { nonce }
            | IpcMessage::Pong { nonce } =>
                {
                    let mut data = vec![0; 5];
                    let nonce_bytes = nonce.to_le_bytes();

                    data[0] = self.id();
                    data[1..5].copy_from_slice(&nonce_bytes);
                    data
                }
        }
    }

    pub fn from_wire(data: &[u8]) -> Result<IpcMessage, IpcError>
    {
        fn expect_length(name: &str, data: &[u8], expected: usize) -> Result<(), IpcError>
        {
            if data.len() != expected
            {
                let message = format!("Message {} expected {} but got {} bytes.",
                                      name,
                                      expected,
                                      data.len());

                return Err(IpcError::DecodeError{ message });
            }

            Ok(())
        }

        if data.is_empty()
        {
            return Err(IpcError::DecodeError{ message: "No data provided.".to_string() });
        }

        match data[0]
        {
            0 =>
                {
                    expect_length("Init", data, 3)?;
                    Ok(IpcMessage::InitAsScriptHost
                        {
                            compile_mode: CompileMode::try_from(data[1])?,
                            script_language: ScriptLanguage::try_from(data[2])?
                        })
                },

            1 =>
                {
                    expect_length("Shutdown", data, 1)?;
                    Ok(IpcMessage::Shutdown)
                },

            2 =>
                {
                    expect_length("Ping", data, 5)?;

                    let nonce = u32::from_le_bytes(data[1..5].try_into().unwrap());
                    Ok(IpcMessage::Ping { nonce })
                }

            3 =>
                {
                    expect_length("Pong", data, 5)?;

                    let nonce = u32::from_le_bytes(data[1..5].try_into().unwrap());
                    Ok(IpcMessage::Pong { nonce })
                }

            _ =>
                {
                    let message = format!("Unknown message id: {}.", data[0]);

                    Err(IpcError::DecodeError{ message })
                }
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcPacket
{
    pub id: i64,
    pub message: IpcMessage
}


impl IpcPacket
{
    pub fn new(id: i64, message: &IpcMessage) -> Self
    {
        IpcPacket { id, message: message.clone() }
    }

    pub fn to_wire(&self) -> Vec<u8>
    {
        let payload = self.message.to_wire();
        let mut packet = vec![0; 8 + payload.len()];

        packet[0..=7].copy_from_slice(&self.id.to_le_bytes());
        packet[8..].copy_from_slice(&payload);

        packet
    }

    pub fn from_wire(data: &[u8]) -> Result<Self, IpcError>
    {
        if data.len() <= 8
        {
            let message = format!("Invalid packet length: {}.", data.len());

            return Err(IpcError::DecodeError{ message });
        }

        let id = i64::from_le_bytes(data[0..=7].try_into().unwrap());
        let message = IpcMessage::from_wire(&data[8..])?;

        Ok(IpcPacket { id, message })
    }

    pub fn sent_from_parent(&self) -> bool
    {
        self.id > 0
    }

    pub fn sent_from_child(&self) -> bool
    {
        self.id < 0
    }

    pub fn is_valid(&self) -> bool
    {
        self.id != 0
    }
}
