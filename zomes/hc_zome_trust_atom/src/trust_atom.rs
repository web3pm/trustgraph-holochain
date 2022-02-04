#![allow(clippy::module_name_repetitions)]

use std::collections::BTreeMap;

use hdk::prelude::*;
use holo_hash::EntryHashB64;

enum LinkDirection {
  Forward,
  Reverse,
}

/// Client-facing representation of a Trust Atom
/// We may support JSON in the future to allow for more complex data structures @TODO
#[derive(Serialize, Deserialize, SerializedBytes, Debug, Clone, PartialEq)]
pub struct TrustAtom {
  pub source: String,
  pub target: String,
  pub content: String,
  pub value: String,
  pub source_entry_hash: EntryHashB64,
  pub target_entry_hash: EntryHashB64,
  pub attributes: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, SerializedBytes, Debug, Clone)]
pub struct TrustAtomInput {
  pub target: EntryHash,
  pub content: String,
  pub value: String,
  pub attributes: BTreeMap<String, String>,
}

#[hdk_entry(id = "restaurant", visibility = "public")]
#[derive(Clone)]
pub struct StringTarget(String);

const UNICODE_NUL: &str = "\u{0}"; // Unicode NUL character
const LINK_TAG_HEADER: &str = "\u{166}"; // "Ŧ"
const LINK_TAG_DIRECTION_FORWARD: &str = "\u{2192}"; // "→"
const LINK_TAG_DIRECTION_BACKWARD: &str = "\u{21a9}"; // "↩"

// #[allow(clippy::missing_panics_doc)]
pub fn create(input: TrustAtomInput) -> ExternResult<()> {
  //   let unicode_nul = std::str::from_utf8(&[0]).expect("Unable to convert Unicode NUL to UTF-8");
  let agent_info = agent_info()?;
  let agent_address: EntryHash = agent_info.agent_initial_pubkey.into();

  let forward_link_tag_string = format!(
    "{}{}{}{}{}",
    LINK_TAG_HEADER,
    LINK_TAG_DIRECTION_FORWARD,
    input.content.clone(),
    UNICODE_NUL,
    input.value
  );
  let forward_link_tag = link_tag(forward_link_tag_string)?;
  // debug!("about to create forward link");
  create_link(
    agent_address.clone(),
    input.target.clone(),
    forward_link_tag,
  )?;

  let backward_link_tag_string = format!(
    "{}{}{}",
    LINK_TAG_HEADER, LINK_TAG_DIRECTION_BACKWARD, input.content
  );
  let backward_link_tag = link_tag(backward_link_tag_string)?;
  create_link(input.target, agent_address, backward_link_tag)?;

  // let trust_atom = TrustAtom {
  //     target: input.target,
  //     content: input.content,
  //     value: input.value,
  //     attributes: input.attributes,
  // };
  // Ok(trust_atom)

  Ok(())
}

pub fn query_mine(
  target: Option<EntryHash>,
  content_starts_with: Option<String>,
  min_rating: Option<String>,
) -> ExternResult<Vec<TrustAtom>> {
  let agent_address: EntryHash = agent_info()?.agent_initial_pubkey.into();

  let result =
    query(Some(agent_address), target, content_starts_with, min_rating)?;

  Ok(result)
}

/// Required: exactly one of source or target
/// All other arguments are optional
/// Arguments act as additive filters (AND)
pub fn query(
  source: Option<EntryHash>,
  target: Option<EntryHash>,
  content_starts_with: Option<String>,
  _min_rating: Option<String>,
) -> ExternResult<Vec<TrustAtom>> {
  // let link_direction: LinkDirection;

  let (link_direction, link_base) = match (source, target) {
    (Some(source), None) => (LinkDirection::Forward, source),
    (None, Some(target)) => (LinkDirection::Reverse, target),
    (None, None) => {
      return Err(WasmError::Guest(
        "Either source or target must be specified".into(),
      ))
    }
    (Some(_source), Some(_target)) => {
      return Err(WasmError::Guest(
        "Exactly one of source or target must be specified, but not both"
          .into(),
      ))
    }
  };

  let link_tag = match content_starts_with {
    Some(content_starts_with) => Some(link_tag(content_starts_with)?),
    None => None,
  };

  let links = get_links(link_base.clone(), link_tag)?;

  let trust_atoms =
    convert_links_to_trust_atoms(links, &link_direction, &link_base)?;

  Ok(trust_atoms)
}

fn convert_links_to_trust_atoms(
  links: Vec<Link>,
  link_direction: &LinkDirection,
  link_base: &EntryHash,
) -> ExternResult<Vec<TrustAtom>> {
  let trust_atoms_result: Result<Vec<TrustAtom>, _> = links
    .into_iter()
    .map(|link| convert_link_to_trust_atom(link, link_direction, link_base))
    .collect();
  let trust_atoms = trust_atoms_result?;
  Ok(trust_atoms)
  // .ok_or_else(|_| WasmError::Guest("Failure in converting links to trust atoms".to_string()))?;
  //   Ok(trust_atoms.or_else(|_| WasmError::Guest("erro"))?)
}

fn convert_link_to_trust_atom(
  link: Link,
  link_direction: &LinkDirection,
  link_base: &EntryHash,
) -> ExternResult<TrustAtom> {
  const HC_LINK_HEADER_BYTES: usize = 1; // always created by HC links
                                         //   let unicode_nul = std::str::from_utf8(&[0]).unwrap();

  let link_tag_bytes =
    link.tag.clone().into_inner()[HC_LINK_HEADER_BYTES..].to_vec();
  let link_tag = match String::from_utf8(link_tag_bytes) {
    Ok(link_tag) => link_tag,
    Err(_) => {
      return Err(WasmError::Guest(format!(
        "Link tag is not valid UTF-8 -- found: {}",
        String::from_utf8_lossy(&link.tag.into_inner())
      )))
    }
  };

  let chunks: Vec<&str> = link_tag.split(UNICODE_NUL).collect();
  let content = chunks[0][tg_link_tag_header_length()..].to_string(); // drop leading "Ŧ→" or "Ŧ↩"
  let value = chunks[1].to_string();

  let link_base_b64 = EntryHashB64::from(link_base.clone());
  let link_target_b64 = EntryHashB64::from(link.target);

  let trust_atom = match link_direction {
    LinkDirection::Forward => {
      TrustAtom {
        source: link_base_b64.to_string(),
        target: link_target_b64.to_string(),
        content,
        value,
        source_entry_hash: link_base_b64,
        target_entry_hash: link_target_b64,
        attributes: BTreeMap::new(), // TODO
      }
    }
    LinkDirection::Reverse => {
      TrustAtom {
        source: "".into(),   // TODO
        target: "".into(),   // TODO
        content: link_tag,   // TODO
        value: "999".into(), // TODO
        source_entry_hash: link_target_b64,
        target_entry_hash: link_base.clone().into(),
        attributes: BTreeMap::new(), // TODO
      }
    }
  };
  Ok(trust_atom)
}

pub fn create_string_target(input: String) -> ExternResult<EntryHash> {
  let string_target = StringTarget(input);

  create_entry(string_target.clone())?;

  let target_entry_hash = hash_entry(string_target)?;
  Ok(target_entry_hash)
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes)]
struct StringLinkTag(String);

pub fn link_tag(tag: String) -> ExternResult<LinkTag> {
  let serialized_bytes: SerializedBytes = StringLinkTag(tag).try_into()?;
  Ok(LinkTag(serialized_bytes.bytes().clone()))
}

const fn tg_link_tag_header_length() -> usize {
  LINK_TAG_HEADER.len() + LINK_TAG_DIRECTION_FORWARD.len()
}
