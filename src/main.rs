// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! A very simple example used as a self-test of this library against a Bitcoin
//! Core node.
extern crate bitcoincore_rpc;

use bitcoin::Network;

use bitcoincore_rpc::{bitcoin, Auth, Client, Error, RpcApi};

use sqlite::{Value};

use std::env;
const LOCKTIME_THRESHOLD:i32 = 500000000;
struct NetworkParams {
    host:           String,
    port:           u16,
    dir_path:       String,
    db_field:       String,
}
fn get_network_params(network:Network) -> NetworkParams{
    match network {
        Network::Testnet    =>  NetworkParams{
            host:           "http://localhost".to_string(),
            port:           18333,
            dir_path:       "testnet/".to_string(),
            db_field:        "testnet".to_string(),
        },
        Network::Signet     =>  NetworkParams{
            host:           "http://localhost".to_string(),
            port:           18333,
            dir_path:       "signet/".to_string(),
            db_field:        "signet".to_string(),
        },
        Network::Regtest    =>  NetworkParams{
            host:           "http://localhost".to_string(),
            port:           18443,
            dir_path:       "regtest/".to_string(),
            db_field:        "regtest".to_string(),
        },
        _                   =>  NetworkParams{
            host:           "http://localhost".to_string(),
            port:           8333,
            dir_path:       "".to_string(),
            db_field:       "mainnet".to_string(),
        },
    }
}
fn main_result() -> Result<(), Error> {
    let mut args = std::env::args();

    let _exe_name = args.next().unwrap();

    /*let url = args.next().expect("Usage: <rpc_url> <username> <password>");
    let user = args.next().expect("no user given");
    let pass = args.next().expect("no pass given");
    */
    let network = Network::Regtest;
    let network_params = get_network_params(network);
    let rpc_user = "rpc_user".to_string();
    let rpc_pass = "rpc_pass".to_string();
    let url = format!("{}:{}",network_params.host,network_params.port);
    let rpc = match env::var_os("HOME") {
        Some(home) => {
            if let Some(home_str) = home.to_str(){
                let cookie_file_path = format!("{}/.bitcoin/{}.cookie",home_str, network_params.dir_path);
                dbg!(&cookie_file_path);
                Client::new(&url[..], Auth::CookieFile(cookie_file_path.into())).unwrap()
            }else{ panic!()}
        }
        None => Client::new(&url[..], Auth::UserPass(rpc_user, rpc_pass)).unwrap() 
    };

    let _blockchain_info = rpc.get_blockchain_info()?;

    let best_block_hash = rpc.get_best_block_hash()?;
    println!("best block hash: {}", best_block_hash);
    let bestblockcount = rpc.get_block_count()?;
    println!("best block height: {}", bestblockcount);
    let best_block_hash_by_height = rpc.get_block_hash(bestblockcount)?;
    println!("best block hash by height: {}", best_block_hash_by_height);
    assert_eq!(best_block_hash_by_height, best_block_hash);

    let bitcoin_block: bitcoin::Block = rpc.get_by_id(&best_block_hash)?;
    println!("best block hash by `get`: {}", bitcoin_block.header.prev_blockhash);
    match rpc.get_by_id::<bitcoin::Transaction>(&bitcoin_block.txdata[0].txid()){
        Ok(bitcoin_tx) => {println!("tx by `get`: {}", bitcoin_tx.txid());}
        Err(_) => {}
    };

 


    let db = sqlite::open("../bal.db").unwrap();
    dbg!(&network_params.db_field);
    let query_tx = db.prepare("SELECT  * FROM tbl_tx WHERE network = :network AND status = :status AND ( locktime < :bestblock_height  OR locktime > :locktime_threshold AND locktime < :bestblock_time);").unwrap().into_iter();
    //let query_tx = db.prepare("SELECT * FROM tbl_tx where status = :status").unwrap().into_iter();
    let mut pushed_txs:Vec<String> = Vec::new();
    let mut invalid_txs:Vec<String> = Vec::new();
    dbg!(LOCKTIME_THRESHOLD);
    dbg!(bitcoin_block.header.time);
    dbg!(bestblockcount);
    dbg!(&network_params.db_field);
    for row in query_tx.bind::<&[(_, Value)]>(&[
        (":locktime_threshold", (LOCKTIME_THRESHOLD as i64).into()),
        (":bestblock_time", (bitcoin_block.header.time as i64).into()),
        (":bestblock_height", (bestblockcount as i64).into()),
        (":network", network_params.db_field.into()),
        (":status", 0.into()),
        ][..])
    .unwrap()
    .map(|row| row.unwrap())
    {
        let tx = row.read::<&str, _>("tx");
        let txid = row.read::<&str, _>("txid");
        println!("to be pushed{}",txid);
        match rpc.send_raw_transaction(tx){
            Ok(o) => {
                println!("tx: {} pusshata PUSHED\n{}",txid,o);
                pushed_txs.push(txid.to_string());
            },
            Err(err) => {
                println!("Error: {}\n{}",err,txid);
                invalid_txs.push(txid.to_string());
            },
        };
    }
    
    if pushed_txs.len() > 0 {
        let _ = db.execute(format!("UPDATE tbl_tx SET status = 1 WHERE txid in ('{}');",pushed_txs.join("','")));
    }
    if invalid_txs.len() > 0 {
        let _ = db.execute(format!("UPDATE tbl_tx SET status = 2 WHERE txid in ('{}');",invalid_txs.join("','")));
    }
    
    Ok(())
}

fn main() {
    main_result().unwrap();
}
