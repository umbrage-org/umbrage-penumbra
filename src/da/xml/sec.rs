/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025-2026 Shomy
*/
use std::io::Cursor;

use crate::core::seccfg::{SecCfgV4, SecCfgV4Algo};
use crate::da::xml::exts::sej;
use crate::da::{DownloadProtocol, Xml};

pub fn parse_seccfg(xml: &mut Xml) -> Option<SecCfgV4> {
    let seccfg = xml.dev_info.get_partition("seccfg")?;
    let progress = |_, _| {};

    let mut seccfg_header = Vec::with_capacity(seccfg.size);
    let cursor = Cursor::new(&mut seccfg_header);

    xml.upload("seccfg", cursor, progress).ok()?;

    // Cut to 200 bytes
    seccfg_header.truncate(200);

    let mut parsed_seccfg = SecCfgV4::parse_header(&seccfg_header).ok()?;
    let hash = parsed_seccfg.get_encrypted_hash();

    for algo in [SecCfgV4Algo::SW, SecCfgV4Algo::HWv4] {
        let dec_hash = match algo {
            SecCfgV4Algo::SW => sej(xml, &hash, false, false).ok()?,
            SecCfgV4Algo::HWv4 => sej(xml, &hash, false, true).ok()?,
            // On V6, if neither SW or HWv4 works, we can't do anything about it
            _ => return None,
        };

        if dec_hash == parsed_seccfg.get_hash() {
            parsed_seccfg.set_algo(algo);
            return Some(parsed_seccfg);
        }
    }

    None
}

pub fn write_seccfg(xml: &mut Xml, seccfg: &mut SecCfgV4) -> Option<[u8; 512]> {
    let enc_hash = match seccfg.get_algo() {
        Some(SecCfgV4Algo::SW) => sej(xml, &seccfg.get_hash(), true, false).ok()?,
        Some(SecCfgV4Algo::HWv4) => sej(xml, &seccfg.get_hash(), true, true).ok()?,
        _ => return None,
    };

    seccfg.set_encrypted_hash(enc_hash.try_into().unwrap_or([0u8; 32]));
    let seccfg_data = seccfg.create().ok()?;

    let progress = |_, _| {};
    let cursor = Cursor::new(&seccfg_data);

    xml.download("seccfg", 200, cursor, progress).ok()?;

    Some(seccfg_data)
}
