use wincode::{Deserialize, SchemaRead, SchemaWrite};

use crate::core::traits::{FromBytes, ToBytes};

#[derive(Default, SchemaRead, FromBytes)]
#[repr(C)]
pub struct PacketLenParams {
    pub write_pkt_len: u32,
    pub read_pkt_len: u32,
}

#[derive(Default, SchemaWrite, ToBytes)]
#[repr(C)]
pub struct FlashOpParams {
    pub storage_type: u32,
    pub partition_type: u32,
    pub addr: u64,
    pub size: u64,
    pub nand_param: [u8; 32],
}

#[derive(SchemaWrite, ToBytes)]
pub struct EnvParams {
    pub da_log_level: u32,
    pub log_channel: u32,
    pub system_os: u32,
    pub ufs_provision: u32,
    pub reserved: u32,
}

#[derive(SchemaWrite, ToBytes)]
pub struct RebootParams {
    /// If set, the device will reboot into the
    /// specified bootup mode.
    pub is_dev_reboot: u32,
    /// WDT timeout
    pub timeout_ms: u32,
    pub async_flag: u32,
    /// The boot mode (Normal, Fastboot...)
    pub bootup: u32,
    /// Whether the Download Bit is set or not,
    /// which will make the device enter download
    /// mode on the next boot if set.
    pub dlbit: u32,
    pub not_reset_rtc_time: u32,
    /// If set, the device will not disconnect the
    /// USB connection during reboot.
    pub not_disconnect_usb: u32,
}

/* Extensions */

#[derive(SchemaWrite, ToBytes)]
#[repr(C)]
pub struct ExtPointerTable {
    pub magic: u32,
    pub uart_base: u32,
    pub reg_devc: u32,
    pub malloc: u32,
    pub free: u32,
    pub mmc_get_card: u32,
}

#[repr(C)]
#[derive(SchemaWrite, ToBytes)]
pub struct DACtx {
    pub sej_base: u32,
    pub tzcc_base: u32,
    pub da2_base: u32,
    pub da2_size: u32,
    pub write_pkt_len: u32,
    pub read_pkt_len: u32,
    pub storage_type: u32,
    pub usb_log: u32,
}

#[derive(SchemaWrite, ToBytes)]
pub struct SejParams {
    /// Length of the data to encrypt.
    pub length: u32,
    /// Whether to encrypt or decrypt the data.
    pub encrypt: bool,
    /// Wether to use HW encryption or SW.
    pub anti_clone: bool,
    /// Used in Legacy HW encryption.
    pub xor: bool,
    /// Use legacy SEJ HW encryption
    pub legacy: bool,
    /// Whether to perform CBC or ECB encryption.
    pub cbc: bool,
    /// The key to use for encryption:
    /// 0: SW Key
    /// 1: HW Key
    /// 2: HW Wrapped Key
    /// 3: RID Key
    /// 4: Custom Key
    /// 5-255: Fallback to SW key
    /// When anti_clone is enabled, this will be
    /// ignored by SEJ
    pub key_id: u8,
    /// What key size to use:
    /// 0: 128-bit key
    /// 1: 192-bit key
    /// 2: 256-bit key
    pub key_sz: u8,
    pub reserved: u8,
}

impl Default for SejParams {
    fn default() -> Self {
        Self {
            length: 0,
            encrypt: false,
            anti_clone: false,
            xor: false,
            legacy: false,
            cbc: true,
            key_id: 0,
            key_sz: 2,
            reserved: 0,
        }
    }
}
