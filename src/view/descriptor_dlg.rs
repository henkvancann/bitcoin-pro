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

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::model::{
    DescriptorContent, DescriptorGenerator, DescriptorTypes, Document,
    SourceType, TrackingKey, UtxoEntry,
};
use crate::view::PubkeySelectDlg;

static UI: &'static str = include_str!("../../ui/descriptor.glade");

#[derive(Debug, Display, From, Error)]
#[display(doc_comments)]
/// Errors from processing descriptor data
pub enum Error {
    /// You must provide a non-empty name
    EmptyName,

    /// You must select key descriptor
    EmptyKey,

    /// You must use at least two unique keys in the multi-signature scheme
    EmptyKeyset,

    /// You must provide a non-empty script source
    EmptyScript,

    /// you need to specify type of the provided script
    SourceTypeRequired,
}

pub struct DescriptorDlg {
    dialog: gtk::Dialog,

    key: Rc<RefCell<Option<TrackingKey>>>,
    keyset: Rc<RefCell<Vec<TrackingKey>>>,
    utxo_set: Rc<RefCell<Vec<UtxoEntry>>>,

    msg_box: gtk::Box,
    msg_label: gtk::Label,
    msg_image: gtk::Image,

    name_entry: gtk::Entry,

    singlesig_radio: gtk::RadioButton,
    multisig_radio: gtk::RadioButton,
    script_radio: gtk::RadioButton,

    singlesig_box: gtk::Box,
    pubkey_entry: gtk::Entry,
    multisig_frame: gtk::Frame,
    pubkey_tree: gtk::TreeView,
    pubkey_store: gtk::ListStore,
    threshold_spin: gtk::SpinButton,
    threshold_adj: gtk::Adjustment,
    script_frame: gtk::Frame,
    script_combo: gtk::ComboBox,
    script_text: gtk::TextView,
    script_buffer: gtk::TextBuffer,

    add_pk_btn: gtk::ToolButton,
    select_pk_btn: gtk::Button,
    insert_pk_btn: gtk::ToolButton,
    remove_pk_btn: gtk::ToolButton,

    bare_check: gtk::CheckButton,
    hash_check: gtk::CheckButton,
    compat_check: gtk::CheckButton,
    segwit_check: gtk::CheckButton,
    taproot_check: gtk::CheckButton,

    lookup_combo: gtk::ComboBox,
    lookup_btn: gtk::Button,
    utxo_tree: gtk::TreeView,
    utxo_store: gtk::ListStore,

    save_btn: gtk::Button,
    cancel_btn: gtk::Button,
}

impl DescriptorDlg {
    pub fn load_glade() -> Result<Rc<Self>, glade::Error> {
        let builder = gtk::Builder::from_string(UI);

        let save_btn = builder.get_object("save")?;
        let cancel_btn = builder.get_object("cancel")?;

        let msg_box = builder.get_object("messageBox")?;
        let msg_image = builder.get_object("messageImage")?;
        let msg_label = builder.get_object("messageLabel")?;

        let name_entry = builder.get_object("nameEntry")?;

        let singlesig_radio = builder.get_object("singlesigRadio")?;
        let singlesig_box = builder.get_object("singlesigBox")?;
        let pubkey_entry = builder.get_object("pubkeyEntry")?;

        let multisig_radio = builder.get_object("multisigRadio")?;
        let multisig_frame = builder.get_object("multisigFrame")?;
        let threshold_spin = builder.get_object("thresholdSpinner")?;
        let threshold_adj = builder.get_object("thresholdAdj")?;
        let pubkey_tree = builder.get_object("pubkeyTree")?;
        let pubkey_store = builder.get_object("pubkeyStore")?;

        let script_radio = builder.get_object("scriptRadio")?;
        let script_frame = builder.get_object("scriptFrame")?;
        let script_combo = builder.get_object("scriptCombo")?;
        let script_text = builder.get_object("scriptText")?;
        let script_buffer = builder.get_object("scriptBuffer")?;

        let select_pk_btn = builder.get_object("selectPubkey")?;
        let add_pk_btn = builder.get_object("addPubkey")?;
        let insert_pk_btn = builder.get_object("insertPubkey")?;
        let remove_pk_btn = builder.get_object("removePubkey")?;

        let bare_check = builder.get_object("bareChk")?;
        let hash_check = builder.get_object("hashChk")?;
        let compat_check = builder.get_object("compatChk")?;
        let segwit_check = builder.get_object("segwitChk")?;
        let taproot_check = builder.get_object("taprootChk")?;

        let lookup_combo = builder.get_object("lookupCombo")?;
        let lookup_btn = builder.get_object("lookupBtn")?;
        let utxo_tree = builder.get_object("utxoTree")?;
        let utxo_store = builder.get_object("utxoStore")?;

        let me = Rc::new(Self {
            dialog: glade_load!(builder, "descriptorDlg")?,

            key: none!(),
            keyset: empty!(),
            utxo_set: empty!(),

            msg_box,
            msg_image,
            msg_label,

            name_entry,

            singlesig_radio,
            singlesig_box,
            multisig_radio,
            script_radio,
            pubkey_entry,
            multisig_frame,
            pubkey_tree,
            pubkey_store,
            threshold_spin,
            threshold_adj,
            script_frame,
            script_combo,
            script_text,
            script_buffer,

            add_pk_btn,
            select_pk_btn,
            insert_pk_btn,
            remove_pk_btn,

            bare_check,
            hash_check,
            compat_check,
            segwit_check,
            taproot_check,

            lookup_combo,
            lookup_btn,
            utxo_tree,
            utxo_store,

            save_btn,
            cancel_btn,
        });

        Ok(me)
    }
}

impl DescriptorDlg {
    pub fn run(
        self: Rc<Self>,
        doc: Rc<RefCell<Document>>,
        on_save: impl Fn(DescriptorGenerator) + 'static,
        on_cancel: impl Fn() + 'static,
    ) {
        let me = self.clone();

        me.update_ui();

        me.select_pk_btn.connect_clicked(
            clone!(@weak me, @strong doc => move |_| {
                let pubkey_dlg = PubkeySelectDlg::load_glade().expect("Must load");
                pubkey_dlg.run(
                    doc.clone(),
                    clone!(@weak me => move |tracking_account| {
                        let key = tracking_account.key;
                        me.pubkey_entry.set_text(&key.to_string());
                        *me.key.borrow_mut() = Some(key);
                    }),
                    || {},
                );
            }),
        );

        me.add_pk_btn.connect_clicked(
            clone!(@weak me, @strong doc => move |_| {
                let pubkey_dlg = PubkeySelectDlg::load_glade().expect("Must load");
                pubkey_dlg.run(
                    doc.clone(),
                    clone!(@weak me, @strong doc => move |tracking_account| {
                        me.pubkey_store.insert_with_values(None, &[0, 1, 2], &[
                            &tracking_account.name(),
                            &tracking_account.details(),
                            &tracking_account.count(),
                        ]);
                        me.keyset.borrow_mut().push(tracking_account.key);
                    }),
                    || {},
                );
            }),
        );

        me.insert_pk_btn.connect_clicked(
            clone!(@weak me, @strong doc => move |_| {
                let pubkey_dlg = PubkeySelectDlg::load_glade().expect("Must load");
                pubkey_dlg.run(
                    doc.clone(),
                    clone!(@weak me => move |tracking_account| {
                        me.script_buffer.insert_at_cursor(&tracking_account.details());
                    }),
                    || {},
                );
            }),
        );

        me.remove_pk_btn.connect_clicked(
            clone!(@weak me, @strong doc => move |_| {
                if let Some((model, iter)) =
                        me.pubkey_tree.get_selection().get_selected() {
                    let key = model
                        .get_value(&iter, 1)
                        .get::<String>()
                        .expect("Must always be parseble")
                        .expect("Key is always present");
                    if let Some(tracking_account) =
                            doc.borrow().tracking_account_by_key(&key) {
                        let pos = me.keyset
                            .borrow()
                            .iter()
                            .position(|k| k == &tracking_account.key)
                            .expect("Key was just found, so position is present");
                        me.keyset.borrow_mut().remove(pos);
                    }
                    me.pubkey_store.remove(&iter);
                }
            }),
        );

        me.cancel_btn
            .connect_clicked(clone!(@weak self as me => move |_| {
                me.dialog.close();
                on_cancel()
            }));

        me.save_btn.connect_clicked(
            clone!(@weak self as me => move |_| match self.descriptor_generator() {
                Ok(descriptor_generator) => {
                    me.dialog.close();
                    on_save(descriptor_generator);
                }
                Err(err) => {
                    me.display_error(err);
                    me.save_btn.set_sensitive(false);
                }
            }),
        );

        me.dialog.run();
        me.dialog.close();
    }

    pub fn descriptor_generator(&self) -> Result<DescriptorGenerator, Error> {
        let content = self.descriptor_content()?;
        let types = self.descriptor_types();

        // TODO: Make sure that types are compatible with the content

        let name = self.name_entry.get_text().to_string();
        if name.is_empty() {
            Err(Error::EmptyName)?;
        }
        Ok(DescriptorGenerator {
            name,
            content,
            types,
        })
    }

    pub fn descriptor_content(&self) -> Result<DescriptorContent, Error> {
        let content = if self.singlesig_radio.get_active() {
            let key = self.key.borrow().clone().ok_or(Error::EmptyKey)?;
            DescriptorContent::SingleSig(key)
        } else if self.multisig_radio.get_active() {
            let keyset = self.keyset.borrow().clone();
            if keyset.len() < 2 {
                Err(Error::EmptyKeyset)?
            }
            let threshold = self.threshold_spin.get_value_as_int() as u8;
            DescriptorContent::MultiSig(threshold, keyset)
        } else {
            let source_type = match self
                .script_combo
                .get_active_id()
                .ok_or(Error::SourceTypeRequired)?
                .as_str()
            {
                "asm" => SourceType::Assembly,
                "hex" => SourceType::Binary,
                "miniscript" => SourceType::Miniscript,
                "policy" => SourceType::Policy,
                _ => Err(Error::SourceTypeRequired)?,
            };
            // TODO: Validate script source
            let script = self.script_buffer.to_string();
            if script.is_empty() {
                Err(Error::EmptyScript)?
            }
            DescriptorContent::LockScript(source_type, script)
        };

        Ok(content)
    }

    pub fn descriptor_types(&self) -> DescriptorTypes {
        DescriptorTypes {
            bare: self.bare_check.get_active(),
            hashed: self.hash_check.get_active(),
            compat: self.compat_check.get_active(),
            segwit: self.segwit_check.get_active(),
            taproot: self.taproot_check.get_active(),
        }
    }

    pub fn display_info(&self, msg: impl ToString) {
        self.msg_label.set_text(&msg.to_string());
        self.msg_image.set_from_icon_name(
            Some("dialog-information"),
            gtk::IconSize::SmallToolbar,
        );
        self.msg_box.set_visible(true);
    }

    pub fn display_error(&self, msg: impl std::error::Error) {
        self.msg_label.set_text(&msg.to_string());
        self.msg_image.set_from_icon_name(
            Some("dialog-error"),
            gtk::IconSize::SmallToolbar,
        );
        self.msg_box.set_visible(true);
    }

    pub fn update_ui(&self) {
        let is_singlesig = self.singlesig_radio.get_active();
        let is_multisig = self.multisig_radio.get_active();
        let is_lockscript = self.script_radio.get_active();

        self.singlesig_box.set_sensitive(is_singlesig);
        self.multisig_frame.set_sensitive(is_multisig);
        self.script_frame.set_sensitive(is_lockscript);

        match self.update_ui_internal() {
            Ok(None) => {
                self.msg_box.set_visible(false);
                self.save_btn.set_sensitive(true);
            }
            Ok(Some(msg)) => {
                self.display_info(msg);
                self.save_btn.set_sensitive(true);
            }
            Err(err) => {
                self.display_error(err);
                self.save_btn.set_sensitive(false);
            }
        }
    }

    pub fn update_ui_internal(&self) -> Result<Option<String>, Error> {
        Ok(None)
    }
}
