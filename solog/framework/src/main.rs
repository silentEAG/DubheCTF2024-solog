use std::env;
use std::io::{BufReader, Read, Write};

use borsh::{to_vec, BorshDeserialize};
use chall::processor::{SologData, SologInstruction};
use sha2::Sha256;
use sol_ctf_framework::ChallengeBuilder;

use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::{system_instruction, system_program};
use solana_program_test::tokio;
use solana_sdk::pubkey::{self, Pubkey};
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::Keypair;
use std::error::Error;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sha2::Digest;
use std::io::BufRead;

use std::net::{TcpListener, TcpStream};

pub struct ProofOfWork {
    pub prefix: String,
}

impl ProofOfWork {
    const DIFFICULTY: usize = 5;
    
    fn new() -> Self {
        let prefix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10) 
        .map(char::from)
        .collect();
        Self { prefix }
    }

    fn verify(&self, nonce: u128) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", self.prefix, nonce));
        let result = hasher.finalize();
        let hex_result = format!("{:x}", result);

        hex_result.starts_with(&"0".repeat(Self::DIFFICULTY))
    }
}

fn get_line<R: Read>(reader: &mut BufReader<R>) -> Result<String, Box<dyn Error>> {
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let ret = line
        .split(':')
        .nth(1)
        .ok_or("invalid input")?
        .trim()
        .to_string();

    Ok(ret)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:1337")?;

    println!("starting server at port 1337!");

    for stream in listener.incoming() {
        if let Err(e) = stream {
            println!("failed: {}", e);
            continue;
        }
        let stream = stream.unwrap();

        tokio::spawn(async {
            println!("handling new user");
            if let Err(err) = handle_connection(stream).await {
                println!("error: {:?}", err);
            }
        });
    }
    Ok(())
}

async fn read_instructions(mut socket: &TcpStream) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let mut buf = [0; 8];
    socket.read_exact(&mut buf)?;
    let len = u64::from_le_bytes(buf);
    let mut buf = vec![0; len as usize];
    socket.read_exact(&mut buf)?;
    let instructions = serde_json::from_slice(&buf)?;
    Ok(instructions)
}

async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {

    // Proof of Work
    let pow = ProofOfWork::new();
    let prefix = pow.prefix.clone();
    writeln!(socket, "prefix: {}", prefix)?;
    let mut reader = BufReader::new(socket.try_clone()?);
    let nonce = get_line(&mut reader)?;
    let nonce = nonce.parse::<u128>()?;
    if !pow.verify(nonce) {
        writeln!(socket, "invalid nonce")?;
        return Ok(());
    }

    let mut builder = ChallengeBuilder::try_from(socket.try_clone().unwrap()).unwrap();

    #[cfg(debug_assertions)]
    assert!(builder.add_program("./chall/target/deploy/chall.so", Some(chall::ID)) == chall::ID);

    #[cfg(not(debug_assertions))]
    assert!(builder.add_program("./chall.so", Some(chall::ID)) == chall::ID);

    let mut chall = builder.build().await;

    let user_keypair = Keypair::new();
    let user = user_keypair.pubkey();

    let admin_keypair = &chall.ctx.payer;
    let admin = admin_keypair.pubkey();

    chall
        .run_ix(system_instruction::transfer(&admin, &user, 100_000_000_000))
        .await?;

    let title = "Hello World";
    let content = "This is first post for solog!";

    let create_post_ix = SologInstruction::CreatePost {
        title: title.as_bytes().to_vec(),
        content: content.as_bytes().to_vec(),
    };

    let (post, _) = Pubkey::find_program_address(
        &[b"post", &admin.to_bytes(), title.as_bytes()],
        &chall::ID,
    );

    chall.run_ix(Instruction {
        program_id: chall::ID,
        accounts: vec![
            AccountMeta::new(admin, true),
            AccountMeta::new(post, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: to_vec(&create_post_ix)?,
    }).await?;

    writeln!(socket, "user: {}", user)?;
    writeln!(socket, "post: {}", post)?;

    let solve_ixs = read_instructions(&socket).await?;

    if solve_ixs.len() > 8 {
        writeln!(socket, "too many instructions")?;
        return Ok(());
    }
    
    for solve_ix in solve_ixs {

        if solve_ix.program_id != chall::ID {
            writeln!(socket, "invalid program id")?;
            return Ok(());
        }

        chall
            .run_ixs_full(&[solve_ix], &[&user_keypair], &user)
            .await?;
    }

    let post_data = chall.ctx.banks_client.get_account(post).await?.unwrap().data;

    // println!("Post Data Length: {}", post_data.len());

    let post_data = SologData::try_from_slice(&post_data)?;
    let post_data = match post_data {
        SologData::Post(post_data) => post_data,
        _ => {
            writeln!(socket, "invalid post data")?;
            return Ok(());
        },
    };

    // println!("post author: {}", post_data.author);

    if post_data.author == user && post_data.collaborators[2] == Pubkey::try_from_slice(&[0xff; 32]).unwrap() {
        writeln!(socket, "congrats!")?;
        if let Ok(flag) = env::var("FLAG") {
            writeln!(socket, "flag: {}", flag)?;
        } else {
            writeln!(socket, "flag not found, please contact admin")?;
        }
    } else {
        writeln!(socket, "nonono")?;
    }
    
    Ok(())
}
