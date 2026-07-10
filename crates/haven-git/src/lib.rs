// crates/haven-git - dual-identity writer for Haven vaults.
//
// Identity is bound at open time. Human edits travel under the configured
// human signer; agent edits travel under `Haven Agent (<model>)` so the two
// provenance streams stay separable for `AGENTS.md §7`. Off-tree user edits
// are never absorbed by Haven-owned atomic commits.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use git2::{ObjectType, Oid, Repository, Signature, Tree};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;
