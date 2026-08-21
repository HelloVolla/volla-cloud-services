pub mod comment;
pub use comment::*;
pub mod post;
use hdi::prelude::*;
pub use post::*;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    Post(Post),
    Comment(Comment),
}

#[derive(Serialize, Deserialize)]
#[hdk_link_types]
pub enum LinkTypes {
    PostUpdates,
    PostToComments,
    AllPosts,
}

// Validation you perform during the genesis process. Nobody else on the network performs it, only you.
// There *is no* access to network calls in this callback
#[hdk_extern]
pub fn genesis_self_check(_data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

// Validation the network performs when you try to join, you can't perform this validation yourself as you are not a member yet.
// There *is* access to network calls in this function
pub fn validate_agent_joining(
    _agent_pub_key: AgentPubKey,
    _membrane_proof: &Option<MembraneProof>,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

// This is the unified validation callback for all entries and link types in this integrity zome
// Below is a match template for all of the variants of `DHT Ops` and entry and link types
// Holochain has already performed the following validation for you:
// - The action signature matches on the hash of its content and is signed by its author
// - The previous action exists, has a lower timestamp than the new action, and incremented sequence number
// - The previous action author is the same as the new action author
// - The timestamp of each action is after the DNA's origin time
// - AgentActivity authorities check that the agent hasn't forked their chain
// - The entry hash in the action matches the entry content
// - The entry type in the action matches the entry content
// - The entry size doesn't exceed the maximum entry size (currently 4MB)
// - Private entry types are not included in the Op content, and public entry types are
// - If the `Op` is an update or a delete, the original action exists and is a `Create` or `Update` action
// - If the `Op` is an update, the original entry exists and is of the same type as the new one
// - If the `Op` is a delete link, the original action exists and is a `CreateLink` action
// - Link tags don't exceed the maximum tag size (currently 1KB)
// - Countersigned entries include an action from each required signer
// You can read more about validation here: https://docs.rs/hdi/latest/hdi/index.html#data-validation
#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::CreateEntry(op_entry) => match op_entry {
            OpEntry::CreateEntry { app_entry, action } => {
                let action = TypedAction {
                    header: action.header,
                    data: EntryCreationData::Create(action.data),
                };
                match app_entry {
                    EntryTypes::Post(post) => validate_create_post(action, post),
                    EntryTypes::Comment(comment) => validate_create_comment(action, comment),
                }
            }
            OpEntry::UpdateEntry { app_entry, action } => {
                let action = TypedAction {
                    header: action.header,
                    data: EntryCreationData::Update(action.data),
                };
                match app_entry {
                    EntryTypes::Post(post) => validate_create_post(action, post),
                    EntryTypes::Comment(comment) => validate_create_comment(action, comment),
                }
            }
            _ => Ok(ValidateCallbackResult::Valid),
        },
        FlatOp::Update(update_entry) => match update_entry {
            OpUpdate::Entry { app_entry, action } => {
                let original_action_hash = action.original_action_address.clone();
                let original_action = must_get_action(original_action_hash.clone())?
                    .action()
                    .to_owned();
                let original_create_action =
                    match TypedAction::<EntryCreationData>::try_from(original_action) {
                        Ok(action) => action,
                        Err(e) => {
                            return Ok(ValidateCallbackResult::Invalid(format!(
                                "Expected original action to create an entry: {e:?}"
                            )));
                        }
                    };
                match app_entry {
                    EntryTypes::Comment(comment) => {
                        let original_app_entry = must_get_valid_record(original_action_hash)?;
                        let original_comment = match Comment::try_from(original_app_entry) {
                            Ok(entry) => entry,
                            Err(e) => {
                                return Ok(ValidateCallbackResult::Invalid(format!(
                                    "Expected to get Comment from Record: {e:?}"
                                )));
                            }
                        };
                        validate_update_comment(
                            action,
                            comment,
                            original_create_action,
                            original_comment,
                        )
                    }
                    EntryTypes::Post(post) => {
                        let original_app_entry = must_get_valid_record(original_action_hash)?;
                        let original_post = match Post::try_from(original_app_entry) {
                            Ok(entry) => entry,
                            Err(e) => {
                                return Ok(ValidateCallbackResult::Invalid(format!(
                                    "Expected to get Post from Record: {e:?}"
                                )));
                            }
                        };
                        validate_update_post(action, post, original_create_action, original_post)
                    }
                }
            }
            _ => Ok(ValidateCallbackResult::Valid),
        },
        FlatOp::Delete(OpDelete { action }) => {
            let original_action_hash = action.deletes_address.clone();
            let original_record = must_get_valid_record(original_action_hash)?;
            let original_record_action = original_record.action().clone();
            let original_action =
                match TypedAction::<EntryCreationData>::try_from(original_record_action) {
                    Ok(action) => action,
                    Err(e) => {
                        return Ok(ValidateCallbackResult::Invalid(format!(
                            "Expected original action to create an entry: {e:?}"
                        )));
                    }
                };
            let app_entry_type = match original_action.entry_type() {
                EntryType::App(app_entry_type) => app_entry_type.clone(),
                _ => {
                    return Ok(ValidateCallbackResult::Valid);
                }
            };
            let entry = match original_record.entry().as_option() {
                Some(entry) => entry,
                None => {
                    return Ok(ValidateCallbackResult::Invalid(
                        "Original record for a delete must contain an entry".to_string(),
                    ));
                }
            };
            let original_app_entry = match EntryTypes::deserialize_from_type(
                app_entry_type.zome_index,
                app_entry_type.entry_index,
                entry,
            )? {
                Some(app_entry) => app_entry,
                None => {
                    return Ok(ValidateCallbackResult::Invalid(
                        "Original app entry must be one of the defined entry types for this zome"
                            .to_string(),
                    ));
                }
            };
            match original_app_entry {
                EntryTypes::Comment(original_comment) => {
                    validate_delete_comment(action, original_action, original_comment)
                }
                EntryTypes::Post(original_post) => {
                    validate_delete_post(action, original_action, original_post)
                }
            }
        }
        FlatOp::Link(OpLink::CreateLink { link_type, action }) => {
            let base_address = action.base_address.clone();
            let target_address = action.target_address.clone();
            let tag = action.tag.clone();
            match link_type {
                LinkTypes::PostUpdates => {
                    validate_create_link_post_updates(action, base_address, target_address, tag)
                }
                LinkTypes::PostToComments => validate_create_link_post_to_comments(
                    action,
                    base_address,
                    target_address,
                    tag,
                ),
                LinkTypes::AllPosts => {
                    validate_create_link_all_posts(action, base_address, target_address, tag)
                }
            }
        }
        FlatOp::Link(OpLink::DeleteLink {
            original_action,
            link_type,
            action,
        }) => {
            let base_address = original_action.base_address.clone();
            let target_address = original_action.target_address.clone();
            let tag = original_action.tag.clone();
            match link_type {
                LinkTypes::PostUpdates => validate_delete_link_post_updates(
                    action,
                    original_action,
                    base_address,
                    target_address,
                    tag,
                ),
                LinkTypes::PostToComments => validate_delete_link_post_to_comments(
                    action,
                    original_action,
                    base_address,
                    target_address,
                    tag,
                ),
                LinkTypes::AllPosts => validate_delete_link_all_posts(
                    action,
                    original_action,
                    base_address,
                    target_address,
                    tag,
                ),
            }
        }
        FlatOp::CreateRecord(store_record) => match store_record {
            OpRecord::CreateEntry { app_entry, action } => {
                let action = TypedAction {
                    header: action.header,
                    data: EntryCreationData::Create(action.data),
                };
                match app_entry {
                    EntryTypes::Post(post) => validate_create_post(action, post),
                    EntryTypes::Comment(comment) => validate_create_comment(action, comment),
                }
            }
            OpRecord::UpdateEntry { app_entry, action } => {
                let original_action_hash = action.original_action_address.clone();
                let original_record = must_get_valid_record(original_action_hash.clone())?;
                let original_action = match TypedAction::<EntryCreationData>::try_from(
                    original_record.action().clone(),
                ) {
                    Ok(action) => action,
                    Err(_) => {
                        return Ok(ValidateCallbackResult::Invalid(
                            "Original action for an update must be a Create or Update action"
                                .to_string(),
                        ));
                    }
                };
                let create_action = TypedAction {
                    header: action.header.clone(),
                    data: EntryCreationData::Update(action.data.clone()),
                };
                match app_entry {
                    EntryTypes::Post(post) => {
                        let result = validate_create_post(create_action, post.clone())?;
                        if let ValidateCallbackResult::Valid = result {
                            let original_post: Option<Post> = original_record
                                .entry()
                                .to_app_option()
                                .map_err(|e| wasm_error!(e))?;
                            let original_post = match original_post {
                                Some(post) => post,
                                None => {
                                    return Ok(ValidateCallbackResult::Invalid(
                                        "The updated entry type must be the same as the original entry type"
                                            .to_string(),
                                    ));
                                }
                            };
                            validate_update_post(action, post, original_action, original_post)
                        } else {
                            Ok(result)
                        }
                    }
                    EntryTypes::Comment(comment) => {
                        let result = validate_create_comment(create_action, comment.clone())?;
                        if let ValidateCallbackResult::Valid = result {
                            let original_comment: Option<Comment> = original_record
                                .entry()
                                .to_app_option()
                                .map_err(|e| wasm_error!(e))?;
                            let original_comment = match original_comment {
                                Some(comment) => comment,
                                None => {
                                    return Ok(ValidateCallbackResult::Invalid(
                                        "The updated entry type must be the same as the original entry type"
                                            .to_string(),
                                    ));
                                }
                            };
                            validate_update_comment(
                                action,
                                comment,
                                original_action,
                                original_comment,
                            )
                        } else {
                            Ok(result)
                        }
                    }
                }
            }
            OpRecord::DeleteEntry { action } => {
                let original_action_hash = action.deletes_address.clone();
                let original_record = must_get_valid_record(original_action_hash)?;
                let original_action = match TypedAction::<EntryCreationData>::try_from(
                    original_record.action().clone(),
                ) {
                    Ok(action) => action,
                    Err(_) => {
                        return Ok(ValidateCallbackResult::Invalid(
                            "Original action for a delete must be a Create or Update action"
                                .to_string(),
                        ));
                    }
                };
                let app_entry_type = match original_action.entry_type() {
                    EntryType::App(app_entry_type) => app_entry_type.clone(),
                    _ => {
                        return Ok(ValidateCallbackResult::Valid);
                    }
                };
                let entry = match original_record.entry().as_option() {
                    Some(entry) => entry,
                    None => {
                        if original_action.entry_type().visibility().is_public() {
                            return Ok(ValidateCallbackResult::Invalid(
                                "Original record for a delete of a public entry must contain an entry"
                                    .to_string(),
                            ));
                        } else {
                            return Ok(ValidateCallbackResult::Valid);
                        }
                    }
                };
                let original_app_entry = match EntryTypes::deserialize_from_type(
                    app_entry_type.zome_index,
                    app_entry_type.entry_index,
                    entry,
                )? {
                    Some(app_entry) => app_entry,
                    None => {
                        return Ok(ValidateCallbackResult::Invalid(
                            "Original app entry must be one of the defined entry types for this zome"
                                .to_string(),
                        ));
                    }
                };
                match original_app_entry {
                    EntryTypes::Post(original_post) => {
                        validate_delete_post(action, original_action, original_post)
                    }
                    EntryTypes::Comment(original_comment) => {
                        validate_delete_comment(action, original_action, original_comment)
                    }
                }
            }
            OpRecord::CreateLink { link_type, action } => {
                let base_address = action.base_address.clone();
                let target_address = action.target_address.clone();
                let tag = action.tag.clone();
                match link_type {
                    LinkTypes::PostUpdates => validate_create_link_post_updates(
                        action,
                        base_address,
                        target_address,
                        tag,
                    ),
                    LinkTypes::PostToComments => validate_create_link_post_to_comments(
                        action,
                        base_address,
                        target_address,
                        tag,
                    ),
                    LinkTypes::AllPosts => validate_create_link_all_posts(
                        action,
                        base_address,
                        target_address,
                        tag,
                    ),
                }
            }
            OpRecord::DeleteLink { action } => {
                let record = must_get_valid_record(action.link_add_address.clone())?;
                let create_link = match TypedAction::<CreateLinkData>::try_from(
                    record.action().clone(),
                ) {
                    Ok(create_link) => create_link,
                    Err(_) => {
                        return Ok(ValidateCallbackResult::Invalid(
                            "The action that a DeleteLink deletes must be a CreateLink"
                                .to_string(),
                        ));
                    }
                };
                let link_type = match LinkTypes::from_type(
                    create_link.zome_index,
                    create_link.link_type,
                )? {
                    Some(lt) => lt,
                    None => {
                        return Ok(ValidateCallbackResult::Valid);
                    }
                };
                let base_address = action.base_address.clone();
                let target_address = create_link.target_address.clone();
                let tag = create_link.tag.clone();
                match link_type {
                    LinkTypes::PostUpdates => validate_delete_link_post_updates(
                        action,
                        create_link,
                        base_address,
                        target_address,
                        tag,
                    ),
                    LinkTypes::PostToComments => validate_delete_link_post_to_comments(
                        action,
                        create_link,
                        base_address,
                        target_address,
                        tag,
                    ),
                    LinkTypes::AllPosts => validate_delete_link_all_posts(
                        action,
                        create_link,
                        base_address,
                        target_address,
                        tag,
                    ),
                }
            }
            OpRecord::CreatePrivateEntry { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::UpdatePrivateEntry { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::CreateCapClaim { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::CreateCapGrant { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::UpdateCapClaim { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::UpdateCapGrant { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::Dna { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::OpenChain { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::CloseChain { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::InitZomesComplete { .. } => Ok(ValidateCallbackResult::Valid),
            _ => Ok(ValidateCallbackResult::Valid),
        },
        FlatOp::AgentActivity(agent_activity) => match agent_activity {
            OpActivity::CreateAgent { agent, action } => {
                let prev = action
                    .prev_action()
                    .ok_or_else(|| {
                        wasm_error!(WasmErrorInner::Guest(
                            "expected a prior action before CreateAgent".into()
                        ))
                    })?
                    .clone();
                let previous_action = must_get_action(prev)?;
                match &previous_action.action().data {
                    ActionData::AgentValidationPkg(AgentValidationPkgData {
                        membrane_proof,
                        ..
                    }) => validate_agent_joining(agent, membrane_proof),
                    _ => Ok(ValidateCallbackResult::Invalid(
                        "The previous action for a `CreateAgent` action must be an `AgentValidationPkg`"
                            .to_string(),
                    )),
                }
            }
            _ => Ok(ValidateCallbackResult::Valid),
        },
    }
}
