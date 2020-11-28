// Bitcoin Pro: Professional bitcoin accounts & assets management
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the AGPL License
// along with this software.
// If not, see <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

use std::collections::BTreeSet;

use lnpbp::bitcoin::{self, Script};

#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub struct DescriptorGenerator {
    pub name: String,
    pub content: DescriptorContent,
    pub types: DescriptorTypes,
}

#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub struct DescriptorTypes {
    pub bare: bool,
    pub hashed: bool,
    pub compat: bool,
    pub segwit: bool,
    pub taproot: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub enum DescriptorContent {
    SingleSig(bitcoin::PublicKey),
    MultiSig(u8, BTreeSet<bitcoin::PublicKey>),
    LockScript(ScriptSource),
}

#[derive(Clone, PartialEq, Eq, Debug, StrictEncode, StrictDecode)]
pub enum ScriptSource {
    Binary(Script),
    Assembly(Script),
    Miniscript(String),
    Policy(String),
}
