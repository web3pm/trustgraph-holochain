
use hdk::prelude::holo_hash::EntryHashB64;
use hdk::prelude::*;

use std::collections::BTreeMap;
use crate::*;

#[derive(Clone, Debug)]
struct RollupData {
  content: String,
  value: String,
  agent_rating: Option<String>,
}

pub fn create_rollup_atoms() -> ExternResult<Vec<TrustAtom>> {
  let me: EntryHash = EntryHash::from(agent_info()?.agent_latest_pubkey);
  let my_atoms: Vec<TrustAtom> = query_mine(None, None, None, None)?;
  // debug!("my_atoms: {:#?}", my_atoms);
  let agents = build_agent_list(my_atoms.clone())?;
  //// for tests only ////
  // let mut vec = Vec::new();
  // for agent in agents.clone() {
  //   let mut vec2 = Vec::new();
  //   let links = get_links(agent.clone(), None)?;
  //   for link in links {
  //     let bytes = link.tag.into_inner();
  //     let tag: Option<String> = String::from_utf8(bytes.clone()).ok();
  //     vec2.push(tag);
  //   }
  //   vec.push(vec2);
  // }
  // debug!("agent_links: {:#?}", vec);
  ////////

  let rollup_silver: BTreeMap<EntryHash, BTreeMap<EntryHash, RollupData>> =
    build_rollup_silver(&me, my_atoms, agents)?;
  let rollup_gold: Vec<TrustAtom> = build_rollup_gold(rollup_silver, me)?;
  Ok(rollup_gold)
}

fn build_agent_list(atoms: Vec<TrustAtom>) -> ExternResult<Vec<EntryHash>> {
  let mut agents: Vec<EntryHash> = Vec::new();
  let _agent_prefix_string = "agent".to_string();
  for atom in atoms {
    let target_hash = EntryHash::from(atom.target_entry_hash);
    if let Some(_agent_prefix_string) = atom.prefix { 
      if !agents.contains(&target_hash) { // prevent duplicates
        agents.push(target_hash); 
      }
    } 
  }
  debug!("agents: {:?}", agents);
  Ok(agents)
}

fn build_rollup_silver(
  me: &EntryHash,
  atoms: Vec<TrustAtom>,
  agents: Vec<EntryHash>,
) -> ExternResult<BTreeMap<EntryHash, BTreeMap<EntryHash, RollupData>>> {
  let mut rollup_silver: BTreeMap<EntryHash, BTreeMap<EntryHash, RollupData>> = BTreeMap::new(); // K: Target (EntryHash) V: BTreeMap<Agent, RollupData>
  let targets: Vec<EntryHash> = atoms
    .into_iter()
    .filter(|ta| ta.prefix != Some("agent".to_string())).map(|x| EntryHash::from(x.target_entry_hash)) // filter out agents
    .collect();
  let link_filter = create_link_tag(&LinkDirection::Reverse, &[None]); // get only TA links
  // debug!("link_filter: {:?}", String::from_utf8_lossy(&target_filter.clone().into_inner()));
  for target in targets.clone() {
    // debug!("target: {:?}", target);
    if !agents.contains(&target) {
      let links = get_links(target.clone(), Some(link_filter.clone()))?;
      let mut links_latest = Vec::new();
      //// tests only ////
      // let mut vec = Vec::new();
      // for link in links.clone() {
      //   let bytes = link.tag.into_inner();
      //   let tag: Option<String> = String::from_utf8(bytes.clone()).ok();
      //   vec.push(tag);
      // }
      // debug!("silver_links: {:#?}", vec);
      ///////
      for link in links.clone() {
        let latest = get_latest(target.clone(), link.target, Some(link_filter.clone()))?;
        if let Some(latest) = latest {
          if !links_latest.contains(&latest) {
            // debug!("latest: {:?}", latest);
            links_latest.push(latest);
          }
        }
      }
      debug!("links_latest: {:?}", links_latest);
      let trust_atoms_latest =
        convert_links_to_trust_atoms(links_latest, &LinkDirection::Reverse, &target)?;
      let mut map: BTreeMap<EntryHash, RollupData> = BTreeMap::new();
      debug!("TA latest: {:#?}", trust_atoms_latest);
      for ta in trust_atoms_latest.clone() {
        let source = EntryHash::from(ta.source_entry_hash);
        if agents.contains(&source) { // get only rated Agent TAs
          if let Some(content) = ta.content {
            if let Some(value) = ta.value {
              // ignore content without a rating

              let chunks = [
                None,
                Some(content.clone()),
              ];

              let filter = create_link_tag(&LinkDirection::Forward, &chunks); // NOTE: filter by content broken if mislabeled
              debug!("tag_filter: {:?}", String::from_utf8_lossy(&filter.clone().into_inner()));
              let agent_rating: Option<String> =
                get_rating(me.clone(), source.clone(), Some(filter))?;
              debug!("agent_rating: {:?}", agent_rating);
              if let Some(rating) = agent_rating {
                if rating.parse::<f64>().unwrap() > 0.0 {
                  // retain only positively rated agents
                  let rollup_data = RollupData {
                    content,
                    value,
                    agent_rating: Some(rating),
                  };
                  debug!("rollup_data: {:?}", rollup_data);
                  map.insert(source.clone(), rollup_data);
                }
              }
            }
          }
        }
      }
      debug!("Map: {:?}", map);
      rollup_silver.insert(target, map);
    }
  }
  debug!("silver: {:#?}", rollup_silver);
  Ok(rollup_silver)
}

fn build_rollup_gold(
  rollup_silver: BTreeMap<EntryHash, BTreeMap<EntryHash, RollupData>>,
  me: EntryHash,
) -> ExternResult<Vec<TrustAtom>> {
  let mut rollup_gold: Vec<TrustAtom> = Vec::new();
  for (target, map) in rollup_silver.clone() {
    let mut sourced_trust_atoms: BTreeMap<String, String> = BTreeMap::new(); // collect to input for rollup extra field
    let mut accumulator: Vec<f64> = Vec::new(); // gather weighted values
    let mut agent_rating_phi_sum: f64 = 0.0; // raised to PHI allows for smooth weight curve
    debug!("map: {:#?}", map);
    for (agent, rollup_data) in map.clone() {
      debug!("data: {:#?}", rollup_data);
      if let Some(rating) = rollup_data.agent_rating {
        agent_rating_phi_sum += (rating.parse::<f64>().expect("Parse Error")).powf(1.618); // could ignore parse err and use .ok() to convert result into option
      }
        let link_latest = get_latest(agent.clone(), target.clone(), None)?;
          if let Some(latest) = link_latest {
          let sourced_atom_latest = convert_link_to_trust_atom(
            latest,
            &LinkDirection::Forward,
            &agent,
          )?;
          sourced_trust_atoms.insert(
            EntryHashB64::from(sourced_atom_latest.source_entry_hash).to_string(),
            EntryHashB64::from(sourced_atom_latest.target_entry_hash).to_string(),
          );
        }
    }
    debug!("phi_sum: {:?}", agent_rating_phi_sum);
    let sourced_atoms: Option<String> = {
      if sourced_trust_atoms.len() > 0 {
        serde_json::to_string(&sourced_trust_atoms).ok() // convert map to JSON
      }
      else { None }
    };
    debug!("sourced_atoms: {:#?}", sourced_atoms);

    for (_agent, rollup_data) in map.clone() {
      if let Some(rating) = rollup_data.agent_rating {
        let calc: f64 = ((rating.parse::<f64>().expect("Parse Error")).powf(1.618) / agent_rating_phi_sum)
          * rollup_data.value.parse::<f64>().expect("Parse Error");
        debug!("calc: {:?}", calc);
        accumulator.push(calc);
      }
    }
    
    let extra_entry_hash: Option<String> = {
      if let Some(atoms) = sourced_atoms {
        Some(create_extra(atoms)?)
      }
      else { None }
    };

    debug!("accum: {:?}", accumulator);
    let my_rating: Option<String> = get_rating(me.clone(), target.clone(), None)?;
    // debug!("my_rating: {:?}", my_rating);
    let weighted_sum: f64 = accumulator.iter().sum();
    debug!("sum: {:?}", weighted_sum);
    let content: Option<String> = {
      let get_latest = get_latest(me.clone(), target.clone(), None)?;
      match get_latest {
        Some(link) => convert_link_to_trust_atom(link, &LinkDirection::Forward, &me)?.content,
        None => None,
      }
    };
    if let Some(rating) = my_rating {
      let parsed: f64 = rating.parse::<f64>().expect("Parse Error");
      let algo: f64 = parsed + ((weighted_sum - parsed) * 0.20); // self weight is 80%
      debug!("algo: {:?}", algo);
      let rollup_atom = create_trust_atom(
        me.clone(),
        target.clone(),
        Some("rollup".to_string()),
        content.clone(),
        Some(algo.to_string()),
        extra_entry_hash.clone()
      )?;
      rollup_gold.push(rollup_atom);
    } else {
      // if no self rating for target then avg the other agents weighted values
      let total = accumulator.len() as f64;
      let algo: f64 = weighted_sum / total;
      let rollup_atom = create_trust_atom(
        me.clone(),
        target.clone(),
        Some("rollup".to_string()),
        content.clone(),
        Some(algo.to_string()),
        extra_entry_hash.clone(),
      )?;
      rollup_gold.push(rollup_atom);
    }
  }
  debug!("gold: {:#?}", rollup_gold);
  Ok(rollup_gold)
}

fn get_rating(
  base: EntryHash,
  target: EntryHash,
  tag_filter: Option<LinkTag>,
) -> ExternResult<Option<String>> {
  let link_latest = get_latest(base.clone(), target, tag_filter)?;
  if let Some(latest) = link_latest {
    let trust_atom_latest = convert_link_to_trust_atom(latest, &LinkDirection::Forward, &base)?;
    // debug!("latest rating: {:?}", trust_atom_latest.value);
    return Ok(trust_atom_latest.value);
  }
  Ok(None)
}

fn get_latest(
  base: EntryHash,
  target: EntryHash,
  tag_filter: Option<LinkTag>,
) -> ExternResult<Option<Link>> {
  let mut links: Vec<Link> = get_links(base, tag_filter)?
    .into_iter()
    .filter(|x| x.target == target)
    .collect();
  // debug!("get_latest_inks: {:?}", links);
  links.sort_by(|a, b| a.timestamp.cmp(&b.timestamp)); // ignoring nanoseconds
  let latest = links.pop();
  // debug!("latest: {:?}", latest);
  match latest {
    Some(link) => Ok(Some(link)),
    None => Ok(None),
  }
}

// fn create_rollup_atoms() {

// rollup_bronze: map = {
//     HIA Entry hash (target): [  // target from my TAs or the rollups of agents in my TG
//
//         {
//             // trust atom:
//             source: zippy
//             value: float
//             content: holochain

//            // plus my rating of the "source" agent
//             agent_rating: float // my rating of zippy on `holochain`
//         }
//     ]
//  }

// rollup_silver: map = {
//     HIA Entry hash (target): [  // target from my TAs or the rollups of agents in my TG
//         {
//             content: [
//                 // latest rating by given agent:
//                  {
//                     source: zippy
//                     value: float
//                     // plus my rating of the "source" agent:
//                     agent_rating: float // my rating of zippy on `holochain`
//               }
//           ]
//        }
//     ]
//  }

// gold:
// rollup_gold: vec<TrustAtom>  = [
//     {
//       source: me
//       type: rollup
//       target: HIA Entry hash:
//       value: float
//       content: holochain
//     }
// ]

// for item in rollup_gold:
//      create_link for each

// }

// ALTERNATE STRATEGY (no agent registry, no ablity to identify whether entry is agentpubkey)

// for TA in my TAs
//   rollup_links = get_links(source: TA.target, type: "rollup")  // returns rollups from agents who have done rollups, but [] from non-agent entries
//
//
