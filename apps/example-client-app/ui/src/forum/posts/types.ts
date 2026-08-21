import type {
  ActionHash,
  AgentPubKey,
  DnaHash,
  EntryHash,
  ExternalHash,
  Record,
  SignedActionHashed,
} from "@holochain/client";

export type PostsSignal = {
  type: "EntryCreated";
  action: SignedActionHashed;
  app_entry: EntryTypes;
} | {
  type: "EntryUpdated";
  action: SignedActionHashed;
  app_entry: EntryTypes;
  original_app_entry: EntryTypes;
} | {
  type: "EntryDeleted";
  action: SignedActionHashed;
  original_app_entry: EntryTypes;
} | {
  type: "LinkCreated";
  action: SignedActionHashed;
  link_type: string;
} | {
  type: "LinkDeleted";
  action: SignedActionHashed;
  link_type: string;
};

/* dprint-ignore-start */
export type EntryTypes =
 | ({ type: 'Comment'; } & Comment)
 | ({  type: 'Post'; } & Post);
/* dprint-ignore-end */

export interface Post {
  title: string;
  content: string;
}

export interface Comment {
  comment: string;
  post_hash: ActionHash;
}
