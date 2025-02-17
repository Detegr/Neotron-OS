//! Block Device related commands for Neotron OS

use crate::{bios, print, println, Ctx, API};

pub static LSBLK_ITEM: menu::Item<Ctx> = menu::Item {
    item_type: menu::ItemType::Callback {
        function: lsblk,
        parameters: &[],
    },
    command: "lsblk",
    help: Some("List all the Block Devices"),
};

pub static READ_ITEM: menu::Item<Ctx> = menu::Item {
    item_type: menu::ItemType::Callback {
        function: read_block,
        parameters: &[
            menu::Parameter::Mandatory {
                parameter_name: "device_idx",
                help: Some("The block device ID to fetch from"),
            },
            menu::Parameter::Mandatory {
                parameter_name: "block_idx",
                help: Some("The block to fetch, 0..num_blocks"),
            },
        ],
    },
    command: "readblk",
    help: Some("List all the Block Devices"),
};

/// Called when the "lsblk" command is executed.
fn lsblk(_menu: &menu::Menu<Ctx>, _item: &menu::Item<Ctx>, _args: &[&str], _ctx: &mut Ctx) {
    let api = API.get();
    let mut found = false;

    println!("Block Devices:");
    for dev_idx in 0..=255u8 {
        if let bios::Option::Some(device_info) = (api.block_dev_get_info)(dev_idx) {
            let (bsize, bunits, dsize, dunits) =
                match device_info.num_blocks * u64::from(device_info.block_size) {
                    x if x < (1024 * 1024 * 1024) => {
                        // Under 1 GiB, give it in 10s of MiB
                        (10 * x / (1024 * 1024), "MiB", x / 100_000, "MB")
                    }
                    x => {
                        // Anything else in GiB
                        (10 * x / (1024 * 1024 * 1024), "GiB", x / 100_000_000, "GB")
                    }
                };
            println!("Device {}:", dev_idx);
            println!("          Name: {}", device_info.name);
            println!("          Type: {:?}", device_info.device_type);
            println!("    Block size: {}", device_info.block_size);
            println!("    Num Blocks: {}", device_info.num_blocks);
            println!(
                "     Card Size: {}.{} {} ({}.{} {})",
                bsize / 10,
                bsize % 10,
                bunits,
                dsize / 10,
                dsize % 10,
                dunits
            );
            println!("     Ejectable: {}", device_info.ejectable);
            println!("     Removable: {}", device_info.removable);
            println!(" Media Present: {}", device_info.media_present);
            println!("     Read Only: {}", device_info.read_only);
            found = true;
        }
    }
    if !found {
        println!("  None");
    }
}

/// Called when the "read_block" command is executed.
fn read_block(_menu: &menu::Menu<Ctx>, _item: &menu::Item<Ctx>, args: &[&str], _ctx: &mut Ctx) {
    let api = API.get();
    let Ok(dev_idx) = args[0].parse::<u8>() else {
        println!("Couldn't parse {:?}", args[0]);
        return;
    };
    let Ok(block_idx) = args[1].parse::<u64>() else {
        println!("Couldn't parse {:?}", args[1]);
        return;
    };
    println!("Reading block {}:", block_idx);
    let mut buffer = [0u8; 512];
    match (api.block_read)(
        dev_idx,
        bios::block_dev::BlockIdx(block_idx),
        1,
        bios::ApiBuffer::new(&mut buffer),
    ) {
        bios::Result::Ok(_) => {
            // Carry on
            let mut count = 0;
            for chunk in buffer.chunks(16) {
                print!("{:03x}: ", count);
                for b in chunk {
                    print!("{:02x} ", *b);
                }
                print!(" ");
                for b in chunk {
                    let c = char::from(*b);
                    print!("{}", if c.is_ascii_graphic() { c } else { '.' });
                }
                count += chunk.len();
                println!();
            }
        }
        bios::Result::Err(e) => {
            println!("Failed to read: {:?}", e);
        }
    }
}
