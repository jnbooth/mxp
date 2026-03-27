use std::ops::{Deref, DerefMut};

use bytestring::ByteString;
use bytestringmut::ByteStringMut;
use log::warn;

use super::fragment::{EntityFragment, MapperFragment, MxpFragment, VariableFragment};
use crate::bytestring_ext::ByteStringMutExt;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutputRegister {
    register: ByteStringMut,
    entity: Option<mxp::Var<ByteString>>,
    parse_as: Option<mxp::ParseAs>,
    variable: ByteStringMut,
}

impl OutputRegister {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_entity<S: AsRef<str>>(&mut self, entity: mxp::Var<S>) {
        self.entity = Some(entity.map_text(|text| self.variable.share(text.as_ref())));
    }

    pub fn set_parse_as(&mut self, parse_as: mxp::ParseAs) {
        self.parse_as = Some(parse_as);
    }

    pub fn set_variable(&mut self, variable: &str) {
        self.variable.push_str(variable);
    }

    pub fn finalize(&mut self, mxp_state: &mut mxp::State) -> Option<MxpFragment> {
        let value = self.register.split().freeze();
        if let Some(parse_as) = self.parse_as.take() {
            return Some(MxpFragment::Mapper(MapperFragment { parse_as, value }));
        }
        let Some(entity) = self.entity.take() else {
            let name = self.variable.split().freeze();
            return Some(MxpFragment::Variable(VariableFragment { name, value }));
        };
        match mxp_state.set_entity(&entity, &value) {
            Ok(Some(entry)) => Some(MxpFragment::Entity(EntityFragment {
                name: entity.name,
                value: entry.value.map(|value| self.variable.share(value)),
                publish: entry.publish,
            })),
            Ok(None) => None,
            Err(e) => {
                warn!(target: "mud.mxp", "{e}");
                None
            }
        }
    }
}

impl Deref for OutputRegister {
    type Target = ByteStringMut;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.register
    }
}

impl DerefMut for OutputRegister {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.register
    }
}
