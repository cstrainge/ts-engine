
use std::fmt::{ self, Debug, Display, Formatter };

use crate::ipc::IpcError;



#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NetworkingCapabilities
{
    pub outbound: bool,
    pub inbound: bool
}



impl NetworkingCapabilities
{
    pub fn any(&self) -> bool
    {
        self.outbound || self.inbound
    }
}



impl Default for NetworkingCapabilities
{
    fn default() -> Self
    {
        Self
        {
            outbound: false,
            inbound: false
        }
    }
}


impl Display for NetworkingCapabilities
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "networking [ ")?;

        if self.outbound { write!(f, "outbound ")?; }
        if self.inbound  { write!(f, "inbound ")?; }

        write!(f, "]")
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FileCapabilities
{
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub delete: bool
}


impl FileCapabilities
{
    pub fn any(&self) -> bool
    {
        self.read || self.write || self.create || self.delete
    }
}


impl Default for FileCapabilities
{
    fn default() -> Self
    {
        Self
        {
            read: false,
            write: false,
            create: false,
            delete: false
        }
    }
}


impl Display for FileCapabilities
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "file [ ")?;

        if self.read   { write!(f, "read ")?; }
        if self.write  { write!(f, "write ")?; }
        if self.create { write!(f, "create ")?; }
        if self.delete { write!(f, "delete ")?; }

        write!(f, "]")
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DirectoryCapabilities
{
    pub enumerate: bool,
    pub create: bool,
    pub delete: bool,
    pub rename: bool
}


impl DirectoryCapabilities
{
    pub fn any(&self) -> bool
    {
        self.enumerate || self.create || self.delete || self.rename
    }
}


impl Default for DirectoryCapabilities
{
    fn default() -> Self
    {
        Self
        {
            enumerate: false,
            create: false,
            delete: false,
            rename: false
        }
    }
}

impl Display for DirectoryCapabilities
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "directory [ ")?;

        if self.enumerate { write!(f, "enumerate ")?; }
        if self.create    { write!(f, "create ")?; }
        if self.delete    { write!(f, "delete ")?; }
        if self.rename    { write!(f, "rename ")?; }

        write!(f, "]")
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SymlinkCapabilities
{
    pub read: bool,
    pub create: bool,
    pub delete: bool
}


impl SymlinkCapabilities
{
    pub fn any(&self) -> bool
    {
        self.read || self.create || self.delete
    }
}


impl Default for SymlinkCapabilities
{
    fn default() -> Self
    {
        Self
        {
            read: false,
            create: false,
            delete: false
        }
    }
}

impl Display for SymlinkCapabilities
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "symlink [ ")?;

        if self.read   { write!(f, "read ")?; }
        if self.create { write!(f, "create ")?; }
        if self.delete { write!(f, "delete ")?; }

        write!(f, "]")
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FilesystemCapabilities
{
    pub files: FileCapabilities,
    pub directories: DirectoryCapabilities,
    pub symlinks: SymlinkCapabilities
}


impl FilesystemCapabilities
{
    pub fn any(&self) -> bool
    {
        self.files.any() || self.directories.any() || self.symlinks.any()
    }
}


impl Default for FilesystemCapabilities
{
    fn default() -> Self
    {
        Self
        {
            files: FileCapabilities::default(),
            directories: DirectoryCapabilities::default(),
            symlinks: SymlinkCapabilities::default()
        }
    }
}


impl Display for FilesystemCapabilities
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "filesystem [ ")?;

        if self.files.any()       { write!(f, "{} ", self.files)?; }
        if self.directories.any() { write!(f, "{} ", self.directories)?; }
        if self.symlinks.any()    { write!(f, "{} ", self.symlinks)?; }

        write!(f, "]")
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Capabilities
{
    pub networking: NetworkingCapabilities,
    pub gpu: bool,
    pub filesystem: FilesystemCapabilities,
    pub subprocess: bool,
    pub audio: bool
}


impl Capabilities
{
    pub fn any(&self) -> bool
    {
        self.networking.any() || self.gpu || self.filesystem.any() || self.subprocess || self.audio
    }

    pub fn to_wire(&self) -> Vec<u8>
    {
        let mut data = vec![0; 16];

        data[0] = self.networking.inbound as u8;
        data[1] = self.networking.outbound as u8;

        data[2] = self.gpu as u8;

        data[3] = self.filesystem.files.create as u8;
        data[4] = self.filesystem.files.delete as u8;
        data[5] = self.filesystem.files.read as u8;
        data[6] = self.filesystem.files.write as u8;

        data[7] = self.filesystem.directories.create as u8;
        data[8] = self.filesystem.directories.delete as u8;
        data[9] = self.filesystem.directories.enumerate as u8;
        data[10] = self.filesystem.directories.rename as u8;

        data[11] = self.filesystem.symlinks.read as u8;
        data[12] = self.filesystem.symlinks.create as u8;
        data[13] = self.filesystem.symlinks.delete as u8;

        data[14] = self.subprocess as u8;
        data[15] = self.audio as u8;

        data
    }

    pub fn from_wire(data: &[u8]) -> Result<Capabilities, IpcError>
    {
        fn decode_bool(byte: u8) ->  Result<bool, IpcError>
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

        if data.len() != 16
        {
            let message = format!("Bad data length, expected 16 bytes, got {}", data.len());

            return Err(IpcError::DecodeError { message });
        }

        let mut capabilities = Capabilities::default();

        capabilities.networking.inbound = decode_bool(data[0])?;
        capabilities.networking.outbound = decode_bool(data[1])?;

        capabilities.gpu = decode_bool(data[2])?;

        capabilities.filesystem.files.create = decode_bool(data[3])?;
        capabilities.filesystem.files.delete = decode_bool(data[4])?;
        capabilities.filesystem.files.read   = decode_bool(data[5])?;
        capabilities.filesystem.files.write  = decode_bool(data[6])?;

        capabilities.filesystem.directories.create    = decode_bool(data[7])?;
        capabilities.filesystem.directories.delete    = decode_bool(data[8])?;
        capabilities.filesystem.directories.enumerate = decode_bool(data[9])?;
        capabilities.filesystem.directories.rename    = decode_bool(data[10])?;

        capabilities.filesystem.symlinks.read   = decode_bool(data[11])?;
        capabilities.filesystem.symlinks.create = decode_bool(data[12])?;
        capabilities.filesystem.symlinks.delete = decode_bool(data[13])?;

        capabilities.subprocess = decode_bool(data[14])?;

        capabilities.audio = decode_bool(data[15])?;

        Ok(capabilities)
    }
}


impl Default for Capabilities
{
    fn default() -> Self
    {
        Self
        {
            networking: NetworkingCapabilities::default(),
            gpu: false,
            filesystem: FilesystemCapabilities::default(),
            subprocess: false,
            audio: false
        }
    }
}


impl Display for Capabilities
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "Capabilities [ ")?;

        if self.networking.any() { write!(f, "{} ", self.networking)?; }
        if self.gpu              { write!(f, "gpu ")?; }
        if self.filesystem.any() { write!(f, "{} ", self.filesystem)?; }
        if self.subprocess       { write!(f, "subprocess ")?; }
        if self.audio            { write!(f, "audio ")?; }

        write!(f, "]")
    }
}


impl Debug for Capabilities
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self)
    }
}



pub fn apply_capabilities(_capabilities: &Capabilities)
{
    //
}
