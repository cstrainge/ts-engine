
use std::{ fmt::{ self, Display, Formatter }, random::random };

use crate::capabilities::Capabilities;


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



#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ScriptLanguage
{
    pub types: bool,
    pub jsx: bool
}


impl ScriptLanguage
{
    pub const JAVASCRIPT: ScriptLanguage = ScriptLanguage { types: false, jsx: false };
    pub const TYPESCRIPT: ScriptLanguage = ScriptLanguage { types: true, jsx: false };
    pub const JSX:        ScriptLanguage = ScriptLanguage { types: false, jsx: true };
    pub const TSX:        ScriptLanguage = ScriptLanguage { types: true, jsx: true };
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcMessage
{
    ApplyCapabilities { capabilities: Capabilities },
    CapabilitiesApplied,

    InitAsScriptHost { compile_mode: CompileMode, script_language: ScriptLanguage },
    ScriptHostReady,

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


pub(crate) fn decode_bool(byte: u8) ->  Result<bool, IpcError>
{
    match byte
    {
        0 => Ok(false),
        1 => Ok(true),

        value =>
            {
                let message = format!("Invalid boolean value: {}.", value);
                Err(IpcError::DecodeError { message })
            }
    }
}



impl IpcMessage
{
    fn id(&self) -> u8
    {
        match self
        {
            IpcMessage::ApplyCapabilities { .. } => 0,
            IpcMessage::CapabilitiesApplied => 1,
            IpcMessage::InitAsScriptHost { .. } => 2,
            IpcMessage::ScriptHostReady => 3,
            IpcMessage::Shutdown => 4,
            IpcMessage::Ping { .. } => 5,
            IpcMessage::Pong { .. } => 6
        }
    }

    pub fn to_wire(&self) -> Vec<u8>
    {
        match self
        {
            IpcMessage::ApplyCapabilities { capabilities } =>
                {
                    let mut data = vec![0; 17];

                    data[0] = self.id();
                    let capabilities_data = capabilities.to_wire();
                    data[1..17].copy_from_slice(&capabilities_data);
                    data
                },

            IpcMessage::CapabilitiesApplied => vec![self.id()],

            IpcMessage::InitAsScriptHost { compile_mode, script_language } =>
                {
                    let mut data = vec![0; 4];

                    data[0] = self.id();
                    data[1] = *compile_mode as u8;
                    data[2] = script_language.types as u8;
                    data[3] = script_language.jsx as u8;

                    data
                },

            IpcMessage::ScriptHostReady => vec![self.id()],

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
                    expect_length("ApplyCapabilities", data, 17)?;
                    let capabilities = Capabilities::from_wire(&data[1..])?;
                    Ok(IpcMessage::ApplyCapabilities { capabilities })
                },

            1 =>
                {
                    expect_length("CapabilitiesApplied", data, 1)?;
                    Ok(IpcMessage::CapabilitiesApplied)
                },

            2 =>
                {
                    expect_length("Init", data, 4)?;
                    Ok(IpcMessage::InitAsScriptHost
                        {
                            compile_mode: CompileMode::try_from(data[1])?,
                            script_language: ScriptLanguage
                                {
                                    types: decode_bool(data[2])?,
                                    jsx: decode_bool(data[3])?
                                }
                        })
                },

            3 =>
                {
                    expect_length("ScriptHostReady", data, 1)?;
                    Ok(IpcMessage::ScriptHostReady)
                },

            4 =>
                {
                    expect_length("Shutdown", data, 1)?;
                    Ok(IpcMessage::Shutdown)
                },

            5 =>
                {
                    expect_length("Ping", data, 5)?;

                    let nonce = u32::from_le_bytes(data[1..5].try_into().unwrap());
                    Ok(IpcMessage::Ping { nonce })
                }

            6 =>
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
