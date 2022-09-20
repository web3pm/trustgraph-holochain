use hdi::prelude::*;
use std::collections::BTreeMap;

#[hdk_entry_helper]
#[derive(Clone)]
pub struct StringTarget(pub String);

#[hdk_entry_helper]
#[derive(Clone)]
pub struct Test {
  pub example_field: String,
  //another_test_field:
}

#[hdk_entry_helper]
#[derive(Clone)]
pub struct Extra {
  pub fields: BTreeMap<String, String>, // extra content
}
#[hdk_entry_defs]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
  #[entry_def]
  Test(Test),
  #[entry_def]
  StringTarget(StringTarget),
  // #[entry_def(required_validations = 5)]
  #[entry_def]
  Extra(Extra), // #[entry_def(name = "hidden_msg", required_validations = 5, visibility = "private")]
                // PrivMsg(PrivMsg)
}

#[hdk_link_types]
pub enum LinkTypes {
  TrustAtom,
  //   Rollup
}
