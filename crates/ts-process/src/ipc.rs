
use std::fmt::{ self, Display, Formatter };



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcMessage
{
    Init,
    Shutdown,
    Ping { value: u32 },
    Pong { value: u32 }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcError
{
    DecodeError{ message: String },
    Timeout
}


impl Display for IpcError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        match self
        {
            IpcError::DecodeError{ message } => write!(f, "IPC decode error: {}", message),
            IpcError::Timeout => write!(f, "IPC timeout error.")
        }
    }
}


impl IpcMessage
{
    fn id(&self) -> u8
    {
        match self
        {
            IpcMessage::Init => 0,
            IpcMessage::Shutdown => 1,
            IpcMessage::Ping { .. } => 2,
            IpcMessage::Pong { .. } => 3,
        }
    }

    pub fn to_wire(&self) -> Vec<u8>
    {
        match self
        {
            IpcMessage::Init => vec![self.id()],

            IpcMessage::Shutdown => vec![self.id()],

              IpcMessage::Ping { value }
            | IpcMessage::Pong { value } =>
                {
                    let mut data = vec![0; 5];
                    let value_bytes = value.to_le_bytes();

                    data[0] = self.id();
                    data[1..5].copy_from_slice(&value_bytes);
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
                    expect_length("Init", data, 1)?;
                    Ok(IpcMessage::Init)
                },

            1 =>
                {
                    expect_length("Shutdown", data, 1)?;
                    Ok(IpcMessage::Shutdown)
                },

            2 =>
                {
                    expect_length("Ping", data, 5)?;

                    let value = u32::from_le_bytes(data[1..5].try_into().unwrap());
                    Ok(IpcMessage::Ping { value })
                }

            3 =>
                {
                    expect_length("Pong", data, 5)?;

                    let value = u32::from_le_bytes(data[1..5].try_into().unwrap());
                    Ok(IpcMessage::Pong { value })
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

    pub fn parent_message(&self) -> bool
    {
        self.id > 0
    }

    pub fn child_message(&self) -> bool
    {
        self.id < 0
    }

    pub fn is_valid(&self) -> bool
    {
        self.id != 0
    }
}
