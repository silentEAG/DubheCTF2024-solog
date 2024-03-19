use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
};

use crate::{
    instruction::POST_SUFFIX,
    processor::{Post, SologData, MAX_COLLABORATORS},
};

pub fn instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: Vec<u8>,
    content: Vec<u8>,
) -> ProgramResult {
    let [author, post_info, system_program] = arrayref::array_ref![accounts, 0, 3];

    if !author.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !post_info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    if system_program.key != &system_program::id() {
        return Err(ProgramError::InvalidAccountData);
    }

    if title.len() > 20 || content.len() > 233 {
        msg!("Post is too long");
        return Err(ProgramError::InvalidInstructionData);
    }

    let (post, post_seed) = Pubkey::find_program_address(
        &[POST_SUFFIX, &author.key.to_bytes(), &title],
        program_id,
    );
    if post_info.key != &post {
        return Err(ProgramError::InvalidAccountData);
    }
    if !post_info.data_is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }

    let post_data = SologData::Post(Post {
        claps: 0,
        author: *author.key,
        collaborator_count: 0,
        comment_count: 0,
        collaborators: [Pubkey::try_from_slice(&[0xff; 32]).unwrap(); MAX_COLLABORATORS],
        title: title.clone(),
        content: content.clone(),
    });

    let post_data_len = to_vec(&post_data)?.len();
    let rent = Rent::default().minimum_balance(post_data_len);

    invoke_signed(
        &system_instruction::create_account(
            author.key,
            post_info.key,
            rent,
            post_data_len as u64,
            program_id,
        ),
        &[author.clone(), post_info.clone(), system_program.clone()],
        &[&[
            POST_SUFFIX,
            &author.key.to_bytes(),
            &title,
            &[post_seed],
        ]],
    )?;

    msg!("Post created at: {}", post_info.key);

    let mut post_info = post_info.try_borrow_mut_data()?;
    post_data.serialize(&mut &mut post_info[..])?;

    Ok(())
}
