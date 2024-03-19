use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
};

use crate::{
    instruction::COMMENT_SUFFIX,
    processor::{Comment, SologData},
};

pub fn instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    content: Vec<u8>,
) -> ProgramResult {
    let [author, post_info, comment_info, system_program] = arrayref::array_ref![accounts, 0, 4];
    
    if !author.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !(post_info.owner == program_id) {
        return Err(ProgramError::InvalidAccountData);
    }
    let post_data = SologData::try_from_slice(&post_info.data.borrow())?;
    let mut post_data = match post_data {
        SologData::Post(post) => post,
        _ => return Err(ProgramError::InvalidInstructionData),
    };

    let (comment, comment_seed) = Pubkey::find_program_address(
        &[
            COMMENT_SUFFIX,
            &post_info.key.to_bytes()[..16],
            &author.key.to_bytes()[..16],
            &post_data.comment_count.to_le_bytes(),
        ],
        program_id,
    );
    if comment_info.key != &comment {
        return Err(ProgramError::InvalidAccountData);
    }
    if !comment_info.data_is_empty() {
        return Err(ProgramError::InvalidAccountData);
    }

    let comment_data = SologData::Comment(Comment {
        claps: 0,
        order: post_data.comment_count,
        author: *author.key,
        content: content.clone(),
    });

    let comment_data_len = to_vec(&comment_data)?.len();
    let rent = Rent::default().minimum_balance(comment_data_len);

    invoke_signed(
        &system_instruction::create_account(
            author.key,
            comment_info.key,
            rent,
            comment_data_len as u64,
            program_id,
        ),
        &[author.clone(), comment_info.clone(), system_program.clone()],
        &[&[
            COMMENT_SUFFIX,
            &post_info.key.to_bytes()[..16],
            &author.key.to_bytes()[..16],
            &post_data.comment_count.to_le_bytes(),
            &[comment_seed],
        ]],
    )?;

    post_data.comment_count += 1;

    let post_key = *post_info.key;
    let comment_order = post_data.comment_count;
    let comment_key = *comment_info.key;

    let mut post_info = post_info.try_borrow_mut_data()?;
    SologData::Post(post_data).serialize(&mut &mut post_info[..])?;

    let mut comment_info = comment_info.try_borrow_mut_data()?;
    comment_data.serialize(&mut &mut comment_info[..])?;

    msg!("Comment {} on Post {} created: {}", comment_order, post_key, comment_key);
    Ok(())
}
