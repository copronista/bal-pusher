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
use serde::Serialize;
use serde::Deserialize;
use std::env;
use std::fs::OpenOptions;
use std::io::{ Write};
use log::{info,debug,trace,warn};

const LOCKTIME_THRESHOLD:i32 = 500000000;

#[derive(Debug, Serialize, Deserialize)]
struct MyConfig {
    address: String,
    fixed_fee: u64,
    bind: String,
    bind_port: u16,
    requests_file: String,
    db_file: String,
    rpc_user: String,
    rpc_pass: String,
    bitcoin_dir: String,


}
impl Default for MyConfig {
    fn default() -> Self {
        MyConfig {
            address: "Unknown".to_string(),
            fixed_fee: 10000,
            bind: "127.0.0.1".to_string(),
            bind_port: 9137,
            requests_file: "rawrequests.log".to_string(),
            db_file: "../bal.db".to_string(),
            rpc_user: "bitcoin".to_string(),
            rpc_pass: "bitcoin".to_string(),
            bitcoin_dir: "".to_string(),
        }
    }
}
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
            port:           18332,
            dir_path:       "testnet3/".to_string(),
            db_field:        "testnet".to_string(),
        },
        Network::Signet     =>  NetworkParams{
            host:           "http://localhost".to_string(),
            port:           18332,
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
            port:           8332,
            dir_path:       "".to_string(),
            db_field:       "mainnet".to_string(),
        },
    }
}
fn get_client(cfg: &MyConfig,network_params:&NetworkParams) -> Result<Client,Error>{
    let url = format!("{}:{}",network_params.host,network_params.port);
    let rpc = match Client::new(&url[..],Auth::UserPass(cfg.rpc_user.to_string(),cfg.rpc_pass.to_string())){
        Ok(client) =>match client.get_best_block_hash(){
            Ok(best_hash) =>{
                info!("connected using rpcuserpass: {}",best_hash);
                client
            }
            Err(err) => {
                dbg!(err);
                warn!("cant connect using username: {} and password: {}",cfg.rpc_user,cfg.rpc_pass);
                match env::var_os("HOME") {
                    Some(home) => {
                        info!("some home {}",home.to_str().unwrap());
                        match home.to_str(){
                            Some(home_str) => {
                                let cookie_file_path = format!("{}/.bitcoin/{}.cookie",home_str, network_params.dir_path);
                                dbg!(&cookie_file_path);
                                dbg!(&url);
                                match Client::new(&url[..], Auth::CookieFile(cookie_file_path.into())) {
                                    Ok(client) =>match client.get_best_block_hash(){
                                        Ok(best_hash) =>{
                                            info!("connected using cookies: {}",best_hash);
                                            client
                                        }
                                        Err(err) => {
                                            dbg!(err);
                                            panic!("impossible to fetch")
                                        }
                                    }
                                    Err(err) => {
                                        panic!("Failed to create client: diocanemaiale {:#?}",err);
                                    }
                                }
                            },
                            None => {
                                panic!("mismatch")
                            }
                        }
                    }
                    None => {
                        panic!("please set HOME environment variable");
                    }   
                }
            }
        }
        Err(err) =>{
            panic!("unable to connect {}",err)
        }
    };
    Ok(rpc)
}

fn main_result() -> Result<(), Error> {
    let mut args = std::env::args();

    let _exe_name = args.next().unwrap();

    /*let url = args.next().expect("Usage: <rpc_url> <username> <password>");
    let user = args.next().expect("no user given");
    let pass = args.next().expect("no pass given");
    */
    //let network = Network::Regtest
    let file = confy::get_configuration_file_path("bal-pusher",None).expect("Error while getting path");
    info!("The configuration file path is: {:#?}", file);
    let cfg: MyConfig = confy::load("bal-pusher",None).expect("cant load config file");
    let arg_network = match args.next(){
        Some(nargs) => nargs,
        None => "main".to_string()
    };

    let network = match arg_network.as_str(){
        "testnet" => Network::Testnet,
        "signet" => Network::Signet,
        "regtest" => Network::Regtest,
        _ => Network::Bitcoin,
    };

    debug!("Network: {}",arg_network);
    let network_params = get_network_params(network);
    trace!("rpc_user: {}",cfg.rpc_user.to_string());
    trace!("rpc_pass: {}",cfg.rpc_pass.to_string());

    match get_client(&cfg,&network_params){
        Ok(rpc) => {
            info!("connected");
            let _blockchain_info = rpc.get_blockchain_info()?;
            let best_block_hash = rpc.get_best_block_hash()?;
            info!("best block hash: {}", best_block_hash);
            let bestblockcount = rpc.get_block_count()?;
            info!("best block height: {}", bestblockcount);
            let best_block_hash_by_height = rpc.get_block_hash(bestblockcount)?;
            info!("best block hash by height: {}", best_block_hash_by_height);
            assert_eq!(best_block_hash_by_height, best_block_hash);
            let from_block= std::cmp::max(0, bestblockcount - 11);
            let mut time_sum:u64=0;
            for i in from_block..bestblockcount{
                let hash = rpc.get_block_hash(i).unwrap();
                let block: bitcoin::Block = rpc.get_by_id(&hash).unwrap();
                //info!("block time: {}", block.header.time);
                time_sum += <u32 as Into<u64>>::into(block.header.time);
                //info!("time_sum:{}",time_sum)
            }
            let average_time = time_sum/11;
            info!("average time: {}",average_time);

            let db = sqlite::open(&cfg.db_file).unwrap();
            
            let query_tx = db.prepare("SELECT  * FROM tbl_tx WHERE network = :network AND status = :status AND ( locktime < :bestblock_height  OR locktime > :locktime_threshold AND locktime < :bestblock_time);").unwrap().into_iter();
            //let query_tx = db.prepare("SELECT * FROM tbl_tx where status = :status").unwrap().into_iter();
            let mut pushed_txs:Vec<String> = Vec::new();
            let mut invalid_txs:Vec<String> = Vec::new();
            for row in query_tx.bind::<&[(_, Value)]>(&[
                (":locktime_threshold", (LOCKTIME_THRESHOLD as i64).into()),
                (":bestblock_time", (average_time as i64).into()),
                (":bestblock_height", (bestblockcount as i64).into()),
                (":network", network_params.db_field.into()),
                (":status", 0.into()),
                ][..])
            .unwrap()
            .map(|row| row.unwrap())
            {
                let tx = row.read::<&str, _>("tx");
                let txid = row.read::<&str, _>("txid");
                let locktime = row.read::<i64,_>("locktime");
                info!("to be pushed: {}: {}",txid, locktime);
                match rpc.send_raw_transaction(tx){
                    Ok(o) => {
                        let mut file = OpenOptions::new()
                            .append(true) // Set the append option
                            .create(true) // Create the file if it doesn't exist
                            .open("valid_txs")?;
                        let data = format!("{}\t:\t{}\t:\t{}\n",txid,average_time,locktime);
                        file.write_all(data.as_bytes())?;
                        drop(file);

                        info!("tx: {} pusshata PUSHED\n{}",txid,o);
                        pushed_txs.push(txid.to_string());
                    },
                    Err(err) => {
                        let mut file = OpenOptions::new()
                            .append(true) // Set the append option
                            .create(true) // Create the file if it doesn't exist
                            .open("invalid_txs")?;
                        let data = format!("{}:\t{}\t:\t{}\t:\t{}\n",txid,err,average_time,locktime);
                        file.write_all(data.as_bytes())?;
                        drop(file);
                        warn!("Error: {}\n{}",err,txid);
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
        }
        Err(_)=>{
            panic!("impossible to get client")
        }
    }
    Ok(())
}

fn main() {
    env_logger::init();
    main_result().unwrap();
}

