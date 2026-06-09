/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025-2026 Shomy
*/
use std::io::{Read, Write};

use log::debug;

use super::structs::FlashOpParams;
use crate::core::FromBytes;
use crate::core::storage::PartitionKind;
use crate::core::traits::ToBytes;
use crate::da::DownloadProtocol;
use crate::da::xflash::XFlash;
use crate::da::xflash::cmds::*;
use crate::da::xflash::structs::PacketLenParams;
use crate::error::{Error, Result};
use crate::le_u64;

pub fn read_flash<W, F>(
    xflash: &mut XFlash,
    addr: u64,
    size: usize,
    section: PartitionKind,
    progress: F,
    writer: W,
) -> Result<()>
where
    W: Write + Send,
    F: FnMut(usize, usize) + Send,
{
    debug!("Reading flash at address {:#X} with size {:#X}", addr, size);

    let storage_type = xflash.get_storage_type() as u32;
    let partition_type = section.as_u32();

    let params = FlashOpParams {
        storage_type,
        partition_type,
        addr,
        size: size as u64,
        ..Default::default()
    };

    xflash.send_cmd(Cmd::ReadData)?;
    xflash.send(&params.to_bytes())?;
    status_ok!(xflash);

    xflash.upload_data(size, writer, progress)?;

    debug!("Flash read completed, 0x{:X} bytes read.", size);

    Ok(())
}

pub fn write_flash<R, F>(
    xflash: &mut XFlash,
    addr: u64,
    size: usize,
    section: PartitionKind,
    reader: R,
    progress: F,
) -> Result<()>
where
    F: FnMut(usize, usize) + Send,
    R: Read + Send,
{
    get_packet_length(xflash)?;

    debug!("Writing flash at address {:#X} with size {:#X}", addr, size);

    let storage_type = xflash.get_storage_type() as u32;
    let partition_type = section.as_u32();

    let params = FlashOpParams {
        storage_type,
        partition_type,
        addr,
        size: size as u64,
        ..Default::default()
    };

    xflash.send_cmd(Cmd::WriteData)?;
    xflash.send(&params.to_bytes())?;

    xflash.download_data(size, reader, progress)?;

    debug!("Flash write completed, 0x{:X} bytes written.", size);

    Ok(())
}

pub fn erase_flash<F>(
    xflash: &mut XFlash,
    addr: u64,
    size: usize,
    section: PartitionKind,
    progress: F,
) -> Result<()>
where
    F: FnMut(usize, usize) + Send,
{
    debug!("Erasing flash at address {:#X} with size {:#X}", addr, size);

    let storage_type = xflash.get_storage_type() as u32;
    let partition_type = section.as_u32();

    let params = FlashOpParams {
        storage_type,
        partition_type,
        addr,
        size: size as u64,
        ..Default::default()
    };

    xflash.send_cmd(Cmd::DeviceCtrl)?;
    xflash.send_cmd(Cmd::StartDlInfo)?;
    status_ok!(xflash);

    xflash.send_cmd(Cmd::Format)?;
    xflash.send(&params.to_bytes())?;

    xflash.progress_report(size, progress)?;

    xflash.send_cmd(Cmd::DeviceCtrl)?;
    xflash.send_cmd(Cmd::EndDlInfo)?;
    status_ok!(xflash);

    debug!("Flash erase completed.");
    Ok(())
}

pub fn download<R, F>(
    xflash: &mut XFlash,
    part_name: &str,
    size: usize,
    reader: R,
    progress: F,
) -> Result<()>
where
    R: Read + Send,
    F: FnMut(usize, usize) + Send,
{
    // Works like write_flash, but instead of address and size, it takes a partition name
    // and writes the whole data to it.
    // The main difference betwen write_flash and this function is that this one
    // relies on the DA to find the partition by name.
    // Also, this command doesn't support writing only a part of the partition,
    // it will always write the whole partition with the data provided.

    xflash.send_cmd(Cmd::DeviceCtrl)?;
    xflash.send_cmd(Cmd::StartDlInfo)?;
    status_ok!(xflash);

    get_packet_length(xflash)?;

    xflash.send_cmd(Cmd::Download)?;
    xflash.send_data(&[part_name.as_bytes(), &size.to_le_bytes()])?;

    debug!("Starting download to partition '{}' with size {:#X}", part_name, size);

    xflash.download_data(size, reader, progress)?;

    xflash.send_cmd(Cmd::DeviceCtrl)?;
    xflash.send_cmd(Cmd::EndDlInfo)?;
    status_ok!(xflash);

    debug!("Download completed, {:#X} bytes sent.", size);

    Ok(())
}

pub fn upload<W, F>(xflash: &mut XFlash, part_name: &str, writer: W, progress: F) -> Result<()>
where
    W: Write + Send,
    F: FnMut(usize, usize) + Send,
{
    xflash.send_cmd(Cmd::Upload)?;
    xflash.send(part_name.as_bytes())?;

    let size = {
        let size_data = xflash.read_data()?;
        status_ok!(xflash);
        if size_data.len() < 8 {
            return Err(Error::proto("Received upload size is too short"));
        }
        le_u64!(size_data, 0) as usize
    };

    debug!("Starting readback of partition '{}'", part_name);

    xflash.upload_data(size, writer, progress)?;

    debug!("Upload completed, 0x{:X} bytes received.", size);

    Ok(())
}

pub fn format<F>(xflash: &mut XFlash, part_name: &str, progress: F) -> Result<()>
where
    F: FnMut(usize, usize) + Send,
{
    let part = xflash.dev_info.get_partition(part_name).ok_or_else(|| {
        Error::proto(format!("Partition '{}' not found in partition table", part_name))
    })?;

    xflash.send_cmd(Cmd::DeviceCtrl)?;
    xflash.send_cmd(Cmd::StartDlInfo)?;
    status_ok!(xflash);

    xflash.send_cmd(Cmd::FormatPartition)?;
    // The device starts sending statuses right after sending the partition name,
    // because MTK forgot to put a status write after the command :/
    // so we have to send it manually through the port and not through send()
    let hdr = xflash.generate_header(part_name.as_bytes());
    xflash.conn.write(&hdr)?;
    xflash.conn.write(part_name.as_bytes())?;

    debug!("Formatting partition '{}'", part_name);

    xflash.progress_report(part.size, progress)?;

    xflash.send_cmd(Cmd::DeviceCtrl)?;
    xflash.send_cmd(Cmd::EndDlInfo)?;
    status_ok!(xflash);

    debug!("Partition '{}' formatted.", part_name);
    Ok(())
}

pub fn set_rsc_info<F, R>(
    xflash: &mut XFlash,
    part_name: &str,
    size: usize,
    mut reader: R,
    mut progress: F,
) -> Result<()>
where
    R: Read,
    F: FnMut(usize, usize),
{
    // Split in chunks of 256 bytes
    // The payload structure is like this:
    // u64 offset LE (each iteration, it increases by 1)
    // 64 bytes partition name (null-terminated)
    // 256 bytes (data)

    let mut offset = 0u64;

    let mut buffer = [0u8; 256];
    let mut payload = [0u8; 328];
    let mut part_name_bytes = [0u8; 64];

    let name_bytes = part_name.as_bytes();
    let name_len = name_bytes.len().min(63);
    part_name_bytes[..name_len].copy_from_slice(&name_bytes[..name_len]);

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let offset_bytes = offset.to_le_bytes();
        payload[1..8].copy_from_slice(&offset_bytes[..7]);
        payload[8..72].copy_from_slice(&part_name_bytes);
        payload[72..328].fill(0); // Better to avoid stale data
        payload[72..72 + bytes_read].copy_from_slice(&buffer[..bytes_read]);

        xflash.devctrl(Cmd::SetRscInfo, Some(&[&payload]))?;

        progress(offset as usize * 256 + bytes_read, size);
        offset += 1;
    }

    Ok(())
}

pub fn get_packet_length(xflash: &mut XFlash) -> Result<(usize, usize)> {
    let packet_length_bytes = xflash.devctrl(Cmd::GetPacketLength, None)?;

    let pkt_len = PacketLenParams::from_bytes(&packet_length_bytes)
        .ok_or_else(|| Error::penumbra("Invalid packet length"))?;

    let write_len = pkt_len.write_pkt_len as usize;
    let read_len = pkt_len.read_pkt_len as usize;

    xflash.write_packet_length = Some(write_len);
    xflash.read_packet_length = Some(read_len);

    Ok((write_len, read_len))
}
