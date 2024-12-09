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
extern crate zmq;
use bitcoin::Network;

use bitcoincore_rpc::{bitcoin, Auth, Client, Error, RpcApi};

use sqlite::{Value};
use serde::Serialize;
use serde::Deserialize;
use std::env;
use std::fs::OpenOptions;
use std::io::{ Write};
use log::{info,debug,warn};
use zmq::{Context, Socket};
use std::str;
use std::{thread, time::Duration};
//use byteorder::{LittleEndian, ReadBytesExt};
//use std::io::Cursor;
use hex;
use std::sync::{Mutex,MutexGuard};
use std::error::Error as StdError;

const LOCKTIME_THRESHOLD:i64 = 5000000;

#[derive(Debug, Clone,Serialize, Deserialize)]
struct MyConfig {
    zmq_listener:   String,
    requests_file:  String,
    db_file:        String,
    bitcoin_dir:    String,
    regtest:        NetworkParams,
    testnet:        NetworkParams,
    signet:         NetworkParams,
    mainnet:        NetworkParams,


}

impl Default for MyConfig {
    fn default() -> Self {
        MyConfig {
            zmq_listener:   "tcp://127.0.0.1:28332".to_string(),
            requests_file:  "rawrequests.log".to_string(),
            db_file:        "../bal.db".to_string(),
            bitcoin_dir:    "".to_string(),
            regtest:        get_network_params(Network::Regtest),
            testnet:        get_network_params(Network::Testnet),
            signet:         get_network_params(Network::Signet),
            mainnet:        get_network_params(Network::Bitcoin),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkParams {
    host:           String,
    port:           u16,
    dir_path:       String,
    db_field:       String,
    cookie_file:    String,
    rpc_user:       String,
    rpc_pass:       String,
}

fn get_network_params(network:Network) -> NetworkParams{
    match network {
        Network::Testnet    =>  NetworkParams{
            host:           "http://localhost".to_string(),
            port:           18332,
            dir_path:       "testnet3/".to_string(),
            db_field:       "testnet".to_string(),
            cookie_file:    "".to_string(),
            rpc_user:       "".to_string(),
            rpc_pass:       "".to_string(),
        },
        Network::Signet     =>  NetworkParams{
            host:           "http://localhost".to_string(),
            port:           18332,
            dir_path:       "signet/".to_string(),
            db_field:        "signet".to_string(),
            cookie_file:    "".to_string(),
            rpc_user:       "".to_string(),
            rpc_pass:       "".to_string(),
        },
        Network::Regtest    =>  NetworkParams{
            host:           "http://localhost".to_string(),
            port:           18443,
            dir_path:       "regtest/".to_string(),
            db_field:        "regtest".to_string(),
            cookie_file:    "".to_string(),
            rpc_user:       "".to_string(),
            rpc_pass:       "".to_string(),
        },
        _                   =>  NetworkParams{
            host:           "http://localhost".to_string(),
            port:           8332,
            dir_path:       "".to_string(),
            db_field:       "bitcoin".to_string(),
            cookie_file:    "".to_string(),
            rpc_user:       "".to_string(),
            rpc_pass:       "".to_string(),
        },
    }
}

fn get_cookie_filename(network: &NetworkParams) ->Result<String,Box<dyn StdError>>{
    if network.cookie_file !=""{
        Ok(network.cookie_file.clone())
    }else{
        match env::var_os("HOME") {
            Some(home) => {
                info!("some home {}",home.to_str().unwrap());
                match home.to_str(){
                    Some(home_str) => {
                        let cookie_file_path = format!("{}/.bitcoin/{}.cookie",home_str, network.dir_path);
                        
                        dbg!(&cookie_file_path);
                        Ok(cookie_file_path)
                    },
                    None => Err("wrong HOME value".into())
                }
            },
            None => Err("Please Set HOME environment variable".into())
        }
    }
}
fn get_client_from_username(url: &String, network: &NetworkParams) -> Result<Client,Box<dyn StdError>>{
    if network.rpc_user != "" {
        match Client::new(&url[..],Auth::UserPass(network.rpc_user.to_string(),network.rpc_pass.to_string())){
            Ok(client) => match client.get_best_block_hash(){
                Ok(_) => Ok(client),
                Err(err) => Err(err.into())
             }
            Err(err)=>Err(err.into())
        }
    }else{
        Err("Failed".into())
    }
}
fn get_client_from_cookie(url: &String,network: &NetworkParams)->Result<Client,Box<dyn StdError>>{
    match get_cookie_filename(network){
        Ok(cookie) => {
            match Client::new(&url[..], Auth::CookieFile(cookie.into())) {
                Ok(client) => match client.get_best_block_hash(){
                    Ok(_) => Ok(client),
                    Err(err) => Err(err.into())
                },
                Err(err)=>Err(err.into())

            }
        },
        Err(err)=>Err(err.into())
    }
}
fn get_client(network: &NetworkParams) -> Result<Client,Box<dyn StdError>>{
    let url = format!("{}:{}",network.host,&network.port);
    match get_client_from_username(&url,network){
        Ok(client) =>{Ok(client)},
        Err(_) =>{
            match get_client_from_cookie(&url,&network){
                Ok(client)=>{
                    Ok(client)
                },
                Err(err)=> Err(err.into())
            }
        }
    }
}
fn main_result(cfg: &MyConfig, network_params: &NetworkParams) -> Result<(), Error> {


    /*let url = args.next().expect("Usage: <rpc_url> <username> <password>");
    let user = args.next().expect("no user given");
    let pass = args.next().expect("no pass given");
    */
    //let network = Network::Regtest
    match get_client(network_params){
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
                time_sum += <u32 as Into<u64>>::into(block.header.time);
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
                (":network", network_params.db_field.clone().into()),
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

fn parse_env(cfg: &Mutex<MyConfig>){
    let mut cfg_lock = cfg.lock().unwrap();
    match env::var("BAL_PUSHER_ZMQ_LISTENER") {
        Ok(value) => {
            cfg_lock.zmq_listener = value;},
        Err(_) => {},
    }
    match env::var("BAL_PUSHER_REQUEST_FILE") {
        Ok(value) => {
            cfg_lock.requests_file = value;},
        Err(_) => {},
    }
    match env::var("BAL_PUSHER_DB_FILE") {
        Ok(value) => {
            cfg_lock.db_file = value;},
        Err(_) => {},
    }
    match env::var("BAL_PUSHER_BITCOIN_DIR") {
        Ok(value) => {
            cfg_lock.bitcoin_dir = value;},
        Err(_) => {},
    }
    cfg_lock = parse_env_netconfig(cfg_lock,"regtest");
    cfg_lock = parse_env_netconfig(cfg_lock,"signet");
    cfg_lock = parse_env_netconfig(cfg_lock,"testnet");
    drop(parse_env_netconfig(cfg_lock,"mainnet"));

}

fn parse_env_netconfig<'a>(mut cfg_lock: MutexGuard<'a, MyConfig>, chain: &'a str) -> MutexGuard<'a, MyConfig>{
    let cfg = match chain{
        "regtest" => &mut cfg_lock.regtest,
        "signet" => &mut cfg_lock.signet,
        "testnet" => &mut cfg_lock.testnet,
        &_ => &mut cfg_lock.mainnet,
    };
    match env::var(format!("BAL_PUSHER_{}_HOST",chain.to_uppercase())) {
        Ok(value) => { cfg.host= value; },
        Err(_) => {},
    }
    match env::var(format!("BAL_PUSHER_{}_PORT",chain.to_uppercase())) {
        Ok(value) => {
            match value.parse::<u64>(){
                Ok(value) =>{ cfg.port = value.try_into().unwrap(); },
                Err(_) => {},
            }
        }
        Err(_) => {},
    }
    match env::var(format!("BAL_PUSHER_{}_DIR_PATH",chain.to_uppercase())) {
        Ok(value) => { cfg.dir_path = value; },
        Err(_) => {},
    }
    match env::var(format!("BAL_PUSHER_{}_DB_FIELD",chain.to_uppercase())) {
        Ok(value) => { cfg.db_field = value; },
        Err(_) => {},
    }
    match env::var(format!("BAL_PUSHER_{}_COOKIE_FILE",chain.to_uppercase())) {
        Ok(value) => { cfg.cookie_file = value; },
        Err(_) => {},
    }
    match env::var(format!("BAL_PUSHER_{}_RPC_USER",chain.to_uppercase())) {
        Ok(value) => { cfg.rpc_user = value; },
        Err(_) => {},
    }
    match env::var(format!("BAL_PUSHER_{}_RPC_PASSWORD",chain.to_uppercase())) {
        Ok(value) => { cfg.rpc_pass = value; },
        Err(_) => {},
    }
    cfg_lock
}

fn main(){
    env_logger::init();

    let file = confy::get_configuration_file_path("bal-pusher",None).expect("Error while getting path");
    info!("The configuration file path is: {:#?}", file);
    let cfg: Mutex<MyConfig> = Mutex::new(confy::load("bal-pusher",None).expect("cant load config file"));
    parse_env(&cfg);
    let mut args = std::env::args();
    let _exe_name = args.next().unwrap();
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
    let cfg_lock = cfg.lock().unwrap();


    debug!("Network: {}",arg_network);
    let network_params = get_network_params(network);

    let context = Context::new();
    let socket: Socket = context.socket(zmq::SUB).unwrap();

    // Indirizzo ZMQ (modifica in base alla tua configurazione)
    let zmq_address = cfg_lock.zmq_listener.clone();  // Sostituisci con il tuo indirizzo ZMQ
    socket.connect(&zmq_address).unwrap();

    // Sottoscriviamo a tutti i messaggi
    socket.set_subscribe(b"").unwrap(); // b"" per ricevere tutti i messaggi
    {
        let cfg = cfg_lock.clone();
        let _ = main_result(&cfg,&network_params);
    }
    info!("In attesa di nuovi blocchi...");

    loop {
        // Ricevi il messaggio
        let message = socket.recv_multipart(0).unwrap();
        let topic = message[0].clone();
        let body = message[1].clone();
        //let seq = message[2].clone();
        //let mut sequence_str = "Unknown".to_string();
        /*if seq.len()==4{
            let mut rdr = Cursor::new(seq);
            let sequence = rdr.read_u32::<LittleEndian>().expect("Failed to read integer");
            sequence_str = sequence.to_string();
        }*/
        if topic == b"hashblock" {
            info!("NEW BLOCK{}", hex::encode(body));  // Stampa il corpo come stringa esadecimale
            let cfg = cfg_lock.clone();
            let _ = main_result(&cfg,&network_params);
        }
        thread::sleep(Duration::from_millis(100)); // Sleep for 100ms
    }
}

