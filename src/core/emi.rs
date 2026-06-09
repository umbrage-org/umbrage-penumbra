/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025-2026 Shomy
*/
use hacc::{Preloader, TryRead};

use crate::error::{Error, Result};

pub fn extract_emi_settings(preloader: &[u8]) -> Result<&[u8]> {
    let preloader = Preloader::try_read(preloader)?;

    preloader.emi().ok_or_else(|| Error::penumbra("Preloader has no EMI settings."))
}
