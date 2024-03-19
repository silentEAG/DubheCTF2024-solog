use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const MAX_COLLABORATORS: usize = 3;
pub const MAX_HEAP_DEV_COUNT: usize = 6;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum HeapCommand {
    Allocate {
        size: u64,
    },
    Edit {
        index: u64,
        data: Vec<u8>,
        resize: bool,
    },
    Search {
        index: u64,
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct HeapKit {
    pub commands: Vec<HeapCommand>
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Post {
    pub claps: u8,
    pub comment_count: u8,

    pub collaborators: [Pubkey; MAX_COLLABORATORS],
    pub collaborator_count: u8,

    pub author: Pubkey,
    pub title: Vec<u8>,
    pub content: Vec<u8>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Comment {
    pub claps: u8,
    pub order: u8,
    pub author: Pubkey,
    pub content: Vec<u8>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum SologData {
    Post(Post),
    Comment(Comment),
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum SologInstruction {
    /// Create a new post
    /// accounts:
    ///  0. author
    ///  1. post pda
    ///  2. system_program
    CreatePost { title: Vec<u8>, content: Vec<u8> },

    /// Add a collaborator to a post
    /// accounts:
    ///  0. author
    ///  1. collaborator
    ///  2. post pda
    AddCollaborator,

    /// Clap for a post
    /// accounts:
    ///  0. viewer
    ///  1. post or comment pda
    ///  2. system_program
    Clap { data: Vec<u8> },

    /// Comment on a post
    /// accounts:
    ///  0. author
    ///  1. post pda
    ///  2. comment pda
    ///  3. system_program
    AddComment { content: Vec<u8> },

    /// Edit a comment
    /// accounts:
    /// 0. author
    /// 1. comment pda
    /// 2. system_program
    EditComment { content: Vec<u8> },
}